//! BlueZ D-Bus transport implementation for Razer Bluetooth headsets.

use std::collections::HashMap;
use tracing::{info, warn};
use zbus::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::protocol::catalog::{RazerModelDescriptor, resolve_model};
use crate::protocol::commands::{
    AutoStandby, RazerLedMode, build_auto_standby_command, build_led_command,
};
use crate::protocol::uuids::{
    RAZER_BLE_COMMAND_UUID, RAZER_BLE_SERVICE_UUID, is_razer_name, is_razer_oui,
};

/// Information about a discovered Razer Bluetooth device in BlueZ.
#[derive(Debug, Clone)]
pub struct DiscoveredRazerDevice {
    /// BlueZ D-Bus object path (e.g. `/org/bluez/hci0/dev_44_5E_CD_11_C4_66`).
    pub object_path: OwnedObjectPath,
    /// Bluetooth MAC address.
    pub address: String,
    /// Advertised device name.
    pub name: String,
    /// BlueZ alias.
    pub alias: String,
    /// Whether the device is currently connected.
    pub connected: bool,
    /// Whether the device is paired.
    pub paired: bool,
    /// Battery percentage from `org.bluez.Battery1`, if exposed.
    pub battery_percentage: Option<u8>,
    /// Resolved Razer product descriptor.
    pub model: RazerModelDescriptor,
    /// GATT characteristic path for `RAZER_BLE_COMMAND_UUID`, if resolved.
    pub command_char_path: Option<OwnedObjectPath>,
}

fn parse_modalias_product_id(modalias: &str) -> Option<u16> {
    let pos = modalias.find('p')?;
    let hex_chars: String = modalias[pos + 1..].chars().take(4).collect();
    if hex_chars.len() == 4 {
        u16::from_str_radix(&hex_chars, 16).ok()
    } else {
        None
    }
}

fn prop_str(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    props
        .get(key)
        .and_then(|v| v.downcast_ref::<&str>().ok())
        .map(str::to_string)
}

fn prop_bool(props: &HashMap<String, OwnedValue>, key: &str) -> bool {
    props
        .get(key)
        .and_then(|v| v.downcast_ref::<bool>().ok())
        .unwrap_or(false)
}

/// Enumerate all Razer Bluetooth audio devices currently known to BlueZ over system D-Bus.
pub async fn enumerate_bluez_razer_devices() -> Result<Vec<DiscoveredRazerDevice>, zbus::Error> {
    let connection = Connection::system().await?;
    let reply = connection
        .call_method(
            Some("org.bluez"),
            "/",
            Some("org.freedesktop.DBus.ObjectManager"),
            "GetManagedObjects",
            &(),
        )
        .await?;

    let managed: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
        reply.body().deserialize()?;

    let mut razer_devices: HashMap<OwnedObjectPath, DiscoveredRazerDevice> = HashMap::new();

    for (path, interfaces) in &managed {
        let Some(device_props) = interfaces.get("org.bluez.Device1") else {
            continue;
        };

        let name = prop_str(device_props, "Name").unwrap_or_default();
        let alias = prop_str(device_props, "Alias").unwrap_or_else(|| name.clone());
        let address = prop_str(device_props, "Address").unwrap_or_default();
        let modalias = prop_str(device_props, "Modalias").unwrap_or_default();

        let is_vendor =
            modalias.contains("v1532") || modalias.contains("v1532".to_ascii_uppercase().as_str());
        let is_oui = is_razer_oui(&address);
        let is_named = is_razer_name(&name) || is_razer_name(&alias);

        let has_razer_service = device_props.get("UUIDs").is_some_and(|v| {
            match &**v {
                Value::Array(arr) => arr.iter().any(|item| {
                    matches!(item, Value::Str(s) if s.eq_ignore_ascii_case(RAZER_BLE_SERVICE_UUID))
                }),
                _ => false,
            }
        });

        if !is_vendor && !is_oui && !is_named && !has_razer_service {
            continue;
        }

        let connected = prop_bool(device_props, "Connected");
        let paired = prop_bool(device_props, "Paired");

        let battery_percentage = interfaces
            .get("org.bluez.Battery1")
            .and_then(|bat_props| bat_props.get("Percentage"))
            .and_then(|v| v.downcast_ref::<u8>().ok());

        let pid = parse_modalias_product_id(&modalias);
        let effective_name = if name.is_empty() { &alias } else { &name };
        let model = resolve_model(effective_name, pid);

        razer_devices.insert(
            path.clone(),
            DiscoveredRazerDevice {
                object_path: path.clone(),
                address,
                name,
                alias,
                connected,
                paired,
                battery_percentage,
                model,
                command_char_path: None,
            },
        );
    }

    // Second pass: resolve GATT command characteristic paths for discovered Razer devices.
    for (path, interfaces) in &managed {
        let Some(char_props) = interfaces.get("org.bluez.GattCharacteristic1") else {
            continue;
        };
        let Some(uuid) = prop_str(char_props, "UUID") else {
            continue;
        };
        if !uuid.eq_ignore_ascii_case(RAZER_BLE_COMMAND_UUID) {
            continue;
        }

        let path_str = path.as_str();
        for (dev_path, dev) in &mut razer_devices {
            if path_str.starts_with(dev_path.as_str()) {
                dev.command_char_path = Some(path.clone());
            }
        }
    }

    Ok(razer_devices.into_values().collect())
}

/// Write a raw command payload to a BlueZ GATT characteristic path.
pub async fn send_razer_packet(
    char_path: &OwnedObjectPath,
    payload: &[u8],
) -> Result<(), zbus::Error> {
    let connection = Connection::system().await?;
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("type", Value::from("command"));

    connection
        .call_method(
            Some("org.bluez"),
            char_path.as_str(),
            Some("org.bluez.GattCharacteristic1"),
            "WriteValue",
            &(payload, options),
        )
        .await?;

    Ok(())
}

/// Active BlueZ GATT client for a Razer Bluetooth headset.
#[derive(Clone)]
pub struct BluezRazerClient {
    /// Discovered BlueZ device metadata.
    pub device: DiscoveredRazerDevice,
}

impl BluezRazerClient {
    /// Wrap a discovered Razer device for issuing GATT commands.
    #[must_use]
    pub const fn new(device: DiscoveredRazerDevice) -> Self {
        Self { device }
    }

    /// Write raw bytes to the Razer BLE command characteristic.
    pub async fn write_raw(&self, payload: &[u8]) -> Result<(), zbus::Error> {
        let Some(ref path) = self.device.command_char_path else {
            warn!(
                "No Razer BLE command characteristic ({}) found for {}",
                RAZER_BLE_COMMAND_UUID, self.device.address
            );
            return Err(zbus::Error::Failure(
                "Missing Razer BLE command characteristic".to_string(),
            ));
        };

        info!(
            "Writing Razer BLE packet to {}: {:02x?}",
            self.device.address, payload
        );
        send_razer_packet(path, payload).await
    }

    /// Set the Razer Chroma BLE LED lighting mode.
    pub async fn set_led_mode(&self, mode: RazerLedMode) -> Result<(), zbus::Error> {
        let packet = build_led_command(mode);
        self.write_raw(&packet).await
    }

    /// Set the auto-standby inactivity timer.
    pub async fn set_auto_standby(&self, duration: AutoStandby) -> Result<(), zbus::Error> {
        let packet = build_auto_standby_command(duration);
        self.write_raw(&packet).await
    }
}

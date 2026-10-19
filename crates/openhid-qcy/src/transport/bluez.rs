//! BlueZ D-Bus transport implementation for QCY Bluetooth devices.

use std::collections::HashMap;
use tracing::{info, warn};
use zbus::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::protocol::catalog::{QcyModelDescriptor, resolve_model};
use crate::protocol::commands::{
    AncMode, build_anc_command, build_auto_power_off_command, build_custom_eq_command,
    build_eq_preset_command, build_game_mode_command, build_ldac_command, build_multipoint_command,
    build_prompt_volume_command,
};
use crate::protocol::uuids::{QCY_COMMAND_UUID, QCY_NOTIFICATION_UUID, QCY_SERVICE_UUID, is_qcy_name};

/// Information about a discovered QCY Bluetooth device in BlueZ.
#[derive(Debug, Clone)]
pub struct DiscoveredQcyDevice {
    pub object_path: OwnedObjectPath,
    pub address: String,
    pub name: String,
    pub alias: String,
    pub connected: bool,
    pub paired: bool,
    pub battery_percentage: Option<u8>,
    pub model: QcyModelDescriptor,
    pub command_char_path: Option<OwnedObjectPath>,
    pub notify_char_path: Option<OwnedObjectPath>,
}

fn parse_modalias_product_id(modalias: &str) -> Option<u16> {
    if let Some(pos) = modalias.find('p') {
        let hex_slice = &modalias[pos + 1..];
        let hex_chars: String = hex_slice.chars().take(4).collect();
        if hex_chars.len() == 4 {
            return u16::from_str_radix(&hex_chars, 16).ok();
        }
    }
    None
}

/// Enumerate all QCY devices currently known to BlueZ over system D-Bus.
pub async fn enumerate_bluez_qcy_devices() -> Result<Vec<DiscoveredQcyDevice>, zbus::Error> {
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

    let mut qcy_devices: HashMap<OwnedObjectPath, DiscoveredQcyDevice> = HashMap::new();

    // First pass: identify QCY devices (org.bluez.Device1)
    for (path, interfaces) in &managed {
        if let Some(device_props) = interfaces.get("org.bluez.Device1") {
            let name = device_props
                .get("Name")
                .and_then(|v| match v.downcast_ref::<&str>() {
                    Ok(s) => Some(s.to_string()),
                    Err(_) => None,
                })
                .unwrap_or_default();

            let alias = device_props
                .get("Alias")
                .and_then(|v| match v.downcast_ref::<&str>() {
                    Ok(s) => Some(s.to_string()),
                    Err(_) => None,
                })
                .unwrap_or_else(|| name.clone());

            let address = device_props
                .get("Address")
                .and_then(|v| match v.downcast_ref::<&str>() {
                    Ok(s) => Some(s.to_string()),
                    Err(_) => None,
                })
                .unwrap_or_default();

            let modalias = device_props
                .get("Modalias")
                .and_then(|v| match v.downcast_ref::<&str>() {
                    Ok(s) => Some(s.to_string()),
                    Err(_) => None,
                })
                .unwrap_or_default();

            let is_vendor = modalias.contains("v05D6") || modalias.contains("v05d6");
            let is_named = is_qcy_name(&name) || is_qcy_name(&alias);

            let has_qcy_service = device_props
                .get("UUIDs")
                .map(|v| match &**v {
                    Value::Array(arr) => arr.iter().any(|item| {
                        if let Value::Str(s) = item {
                            s.as_str().eq_ignore_ascii_case(QCY_SERVICE_UUID)
                        } else {
                            false
                        }
                    }),
                    _ => false,
                })
                .unwrap_or(false);

            if is_vendor || is_named || has_qcy_service {
                let connected = device_props
                    .get("Connected")
                    .and_then(|v| match v.downcast_ref::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => None,
                    })
                    .unwrap_or(false);

                let paired = device_props
                    .get("Paired")
                    .and_then(|v| match v.downcast_ref::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => None,
                    })
                    .unwrap_or(false);

                let battery_percentage = interfaces
                    .get("org.bluez.Battery1")
                    .and_then(|b| b.get("Percentage"))
                    .and_then(|v| match v.downcast_ref::<u8>() {
                        Ok(p) => Some(p),
                        Err(_) => None,
                    });

                let product_id = parse_modalias_product_id(&modalias);
                let model = resolve_model(&name, &alias, product_id);

                qcy_devices.insert(
                    path.clone(),
                    DiscoveredQcyDevice {
                        object_path: path.clone(),
                        address,
                        name,
                        alias,
                        connected,
                        paired,
                        battery_percentage,
                        model,
                        command_char_path: None,
                        notify_char_path: None,
                    },
                );
            }
        }
    }

    // Second pass: associate GATT characteristics (org.bluez.GattCharacteristic1)
    for (path, interfaces) in &managed {
        if let Some(char_props) = interfaces.get("org.bluez.GattCharacteristic1") {
            let uuid = char_props
                .get("UUID")
                .and_then(|v| match v.downcast_ref::<&str>() {
                    Ok(s) => Some(s.to_ascii_lowercase()),
                    Err(_) => None,
                })
                .unwrap_or_default();

            for (device_path, dev) in qcy_devices.iter_mut() {
                if path.as_str().starts_with(device_path.as_str()) {
                    if uuid == QCY_COMMAND_UUID {
                        dev.command_char_path = Some(path.clone());
                    } else if uuid == QCY_NOTIFICATION_UUID {
                        dev.notify_char_path = Some(path.clone());
                    }
                }
            }
        }
    }

    Ok(qcy_devices.into_values().collect())
}

/// Send a raw command packet to the QCY device over its GATT command characteristic.
pub async fn send_qcy_packet(
    command_char_path: &OwnedObjectPath,
    packet: &[u8],
) -> Result<(), zbus::Error> {
    let connection = Connection::system().await?;
    let mut options = HashMap::new();
    options.insert("type", Value::from("request"));

    connection
        .call_method(
            Some("org.bluez"),
            command_char_path,
            Some("org.bluez.GattCharacteristic1"),
            "WriteValue",
            &(packet, options),
        )
        .await?;

    Ok(())
}

/// Unified client handle for interacting with any connected QCY device.
///
/// Shares the single underlying GATT backend across all product families
/// (H3S, H3, H4, MeloBuds, T13, etc.), gating commands against each model's
/// declared hardware capabilities.
#[derive(Clone)]
pub struct BluezQcyClient {
    pub device: DiscoveredQcyDevice,
}

impl BluezQcyClient {
    pub fn new(device: DiscoveredQcyDevice) -> Self {
        Self { device }
    }

    /// Set Noise Cancellation / Transparency mode.
    pub async fn set_anc(&self, mode: AncMode) -> Result<(), zbus::Error> {
        if !self.device.model.capabilities.has_anc {
            warn!(model = %self.device.model.marketing_name, "ANC is not supported on this model");
            return Ok(());
        }

        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_anc_command(mode);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, model = %self.device.model.marketing_name, ?mode, "sent QCY ANC mode");
        } else {
            warn!(address = %self.device.address, "QCY command characteristic not available");
        }
        Ok(())
    }

    /// Set Game Mode (low latency).
    pub async fn set_game_mode(&self, enabled: bool) -> Result<(), zbus::Error> {
        if !self.device.model.capabilities.has_game_mode {
            warn!(model = %self.device.model.marketing_name, "Game mode is not supported on this model");
            return Ok(());
        }

        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_game_mode_command(enabled);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, model = %self.device.model.marketing_name, enabled, "sent QCY game mode");
        } else {
            warn!(address = %self.device.address, "QCY command characteristic not available");
        }
        Ok(())
    }

    /// Set Multipoint (Dual-Device connection).
    pub async fn set_multipoint(&self, enabled: bool) -> Result<(), zbus::Error> {
        if !self.device.model.capabilities.has_multipoint {
            warn!(model = %self.device.model.marketing_name, "Multipoint is not supported on this model");
            return Ok(());
        }

        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_multipoint_command(enabled);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, model = %self.device.model.marketing_name, enabled, "sent QCY multipoint");
        }
        Ok(())
    }

    /// Set LDAC codec toggle.
    pub async fn set_ldac(&self, enabled: bool) -> Result<(), zbus::Error> {
        if !self.device.model.capabilities.has_ldac {
            warn!(model = %self.device.model.marketing_name, "LDAC is not supported on this model");
            return Ok(());
        }

        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_ldac_command(enabled);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, model = %self.device.model.marketing_name, enabled, "sent QCY LDAC");
        }
        Ok(())
    }

    /// Set Prompt Volume level (0..15).
    pub async fn set_prompt_volume(&self, volume: u8) -> Result<(), zbus::Error> {
        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_prompt_volume_command(volume);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, volume, "sent QCY prompt volume");
        }
        Ok(())
    }

    /// Set Auto Power-Off timer in minutes.
    pub async fn set_auto_power_off(&self, minutes: u16) -> Result<(), zbus::Error> {
        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_auto_power_off_command(minutes);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, minutes, "sent QCY auto power-off timer");
        }
        Ok(())
    }

    /// Select an Equalizer preset index.
    pub async fn set_eq_preset(&self, preset_index: u8) -> Result<(), zbus::Error> {
        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_eq_preset_command(preset_index);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, preset_index, "sent QCY EQ preset");
        }
        Ok(())
    }

    /// Set 10-band custom parametric Equalizer gains.
    pub async fn set_custom_eq(&self, gains: &[f64], preset_index: u8) -> Result<(), zbus::Error> {
        if let Some(cmd_path) = &self.device.command_char_path {
            let packet = build_custom_eq_command(gains, preset_index);
            send_qcy_packet(cmd_path, &packet).await?;
            info!(address = %self.device.address, "sent QCY 10-band custom EQ");
        }
        Ok(())
    }
}

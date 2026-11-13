use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

/// Bluetooth inspection, GATT probing, and packet testing tools for headset development.
#[derive(Debug, Subcommand)]
pub enum BtCmd {
    /// List BlueZ adapters and Bluetooth devices with VID/PID, battery, and UUID classification.
    List(ListBtArgs),
    /// Dump the full GATT service/characteristic/descriptor tree of a Bluetooth device.
    Gatt(GattArgs),
    /// Read a GATT characteristic by UUID and print hex + ASCII.
    Read(ReadArgs),
    /// Write raw hex bytes to a GATT characteristic by UUID.
    Write(WriteArgs),
    /// Subscribe to GATT notifications and stream incoming packets as hex + ASCII.
    Watch(WatchArgs),
    /// Capture a JSON snapshot of a Bluetooth device's properties and GATT tree.
    Snapshot(SnapshotArgs),
}

#[derive(Debug, Args)]
pub struct ListBtArgs {
    /// Only show devices advertising audio or vendor headset profiles.
    #[arg(long)]
    pub audio_only: bool,
    /// Filter devices by case-insensitive substring on name, alias, or MAC address.
    #[arg(short, long)]
    pub filter: Option<String>,
    /// Emit JSON output for scripting and fixtures.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct GattArgs {
    /// Bluetooth MAC address (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
    /// Actively read every characteristic that advertises the `read` flag.
    #[arg(long)]
    pub read: bool,
    /// Emit JSON output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ReadArgs {
    /// Characteristic UUID (full 128-bit or 16-bit prefix).
    pub char_uuid: String,
    /// Bluetooth MAC address (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct WriteArgs {
    /// Characteristic UUID (full 128-bit or 16-bit prefix).
    pub char_uuid: String,
    /// Hex payload bytes (e.g. "c30003ff0000" or "c3 00 03 ff 00 00").
    pub hex_bytes: String,
    /// Bluetooth MAC address (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
    /// Use write-with-response (`request`) instead of write-without-response (`command`).
    #[arg(long)]
    pub with_response: bool,
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    /// Bluetooth MAC address (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
    /// Characteristic UUID to watch (defaults to all notify/indicate characteristics).
    #[arg(short, long)]
    pub char_uuid: Option<String>,
    /// Duration in seconds to listen before stopping (0 = listen until Ctrl-C).
    #[arg(short, long, default_value_t = 15)]
    pub duration: u64,
}

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    /// Bluetooth MAC address (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
    /// Write JSON snapshot to a file instead of stdout.
    #[arg(short, long)]
    pub out: Option<PathBuf>,
    /// Redact the lower 3 octets of the MAC address (keeps IEEE OUI prefix).
    #[arg(long)]
    pub redact_mac: bool,
    /// Actively read readable characteristics while capturing the snapshot.
    #[arg(long)]
    pub read: bool,
}

/// Parsed USB/Bluetooth modalias identifiers (`bluetooth:vXXXXpYYYYdZZZZ`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModaliasIds {
    pub source: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: Option<u16>,
}

/// JSON-serializable snapshot of a Bluetooth device and its GATT tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtDeviceSnapshot {
    pub address: String,
    pub name: String,
    pub alias: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
    pub rssi: Option<i16>,
    pub battery_percentage: Option<u8>,
    pub modalias: Option<ModaliasIds>,
    pub uuids: Vec<BtUuidEntry>,
    pub services: Vec<BtGattServiceSnapshot>,
}

/// Classified UUID entry in a device's advertised/resolved UUID list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtUuidEntry {
    pub uuid: String,
    pub label: String,
    pub is_vendor: bool,
}

/// Snapshot of a BlueZ `GattService1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtGattServiceSnapshot {
    pub uuid: String,
    pub label: String,
    pub primary: bool,
    pub characteristics: Vec<BtGattCharSnapshot>,
}

/// Snapshot of a BlueZ `GattCharacteristic1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtGattCharSnapshot {
    pub uuid: String,
    pub label: String,
    pub flags: Vec<String>,
    pub value_hex: Option<String>,
    pub descriptors: Vec<BtGattDescSnapshot>,
}

/// Snapshot of a BlueZ `GattDescriptor1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtGattDescSnapshot {
    pub uuid: String,
    pub value_hex: Option<String>,
}

/// Parse a flexible hex string (`"c3 00 03"`, `"c3:00:03"`, `"0xc3,0x00"`, `"c30003"`) into bytes.
pub fn parse_hex_bytes(input: &str) -> Result<Vec<u8>> {
    let mut cleaned = String::with_capacity(input.len());
    for token in input.split(|c: char| c.is_whitespace() || c == ':' || c == ',' || c == '-') {
        if token.is_empty() {
            continue;
        }
        let stripped = token
            .strip_prefix("0x")
            .or_else(|| token.strip_prefix("0X"))
            .unwrap_or(token);
        cleaned.push_str(stripped);
    }

    if cleaned.is_empty() {
        bail!("empty hex payload");
    }
    if !cleaned.len().is_multiple_of(2) {
        bail!("hex payload must have an even number of nibbles (got {})", cleaned.len());
    }

    let mut out = Vec::with_capacity(cleaned.len() / 2);
    for idx in (0..cleaned.len()).step_by(2) {
        let byte = u8::from_str_radix(&cleaned[idx..idx + 2], 16)
            .with_context(|| format!("invalid hex byte '{}'", &cleaned[idx..idx + 2]))?;
        out.push(byte);
    }
    Ok(out)
}

/// Format a byte slice as space-separated lowercase hex plus printable ASCII preview.
#[must_use]
pub fn format_hex_ascii(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "<empty>".to_string();
    }
    let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let ascii: String = bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                char::from(b)
            } else {
                '.'
            }
        })
        .collect();
    format!("{}  |{}|", hex.join(" "), ascii)
}

/// Redact the device-specific lower 3 octets of a MAC address while preserving the IEEE OUI.
#[must_use]
pub fn redact_mac(mac: &str) -> String {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() == 6 {
        format!("{}:{}:{}:00:00:00", parts[0], parts[1], parts[2])
    } else {
        "00:00:00:00:00:00".to_string()
    }
}

/// Parse a BlueZ `Modalias` string such as `bluetooth:v05D6p000Ad0100` or `usb:v1532p0544d0100`.
#[must_use]
pub fn parse_modalias(modalias: &str) -> Option<ModaliasIds> {
    let (source, rest) = modalias.split_once(':')?;
    let v_pos = rest.find('v')?;
    let p_pos = rest[v_pos + 1..].find('p').map(|i| v_pos + 1 + i)?;
    let vendor_hex = rest.get(v_pos + 1..v_pos + 5)?;
    let product_hex = rest.get(p_pos + 1..p_pos + 5)?;
    let vendor_id = u16::from_str_radix(vendor_hex, 16).ok()?;
    let product_id = u16::from_str_radix(product_hex, 16).ok()?;
    let version = rest[p_pos + 5..]
        .find('d')
        .and_then(|d_rel| {
            let d_pos = p_pos + 5 + d_rel;
            rest.get(d_pos + 1..d_pos + 5)
        })
        .and_then(|hex| u16::from_str_radix(hex, 16).ok());

    Some(ModaliasIds {
        source: source.to_string(),
        vendor_id,
        product_id,
        version,
    })
}

/// Classify a Bluetooth GATT or SDP UUID into a human-readable profile/vendor label
/// and whether it is vendor-specific (`true`) or standard Bluetooth SIG (`false`).
#[must_use]
pub fn classify_uuid(uuid: &str) -> (&'static str, bool) {
    let lower = uuid.to_ascii_lowercase();
    match lower.as_str() {
        // Vendor-specific headset / peripheral services
        "0000a001-0000-1000-8000-00805f9b34fb" => ("QCY BLE Service", true),
        "00001001-0000-1000-8000-00805f9b34fb" => ("QCY Command Characteristic", true),
        "00001002-0000-1000-8000-00805f9b34fb" => ("QCY Notification Characteristic", true),
        "416d656c-2d52-617a-6572-424c4501f40a" => ("Razer BLE Audio Service", true),
        "416d0000-2d52-617a-6572-424c4501f40a" => ("Razer BLE Command Characteristic", true),
        "00010000-0000-1000-8000-011f2000046d" => ("Logitech HID++ BLE Service", true),
        // Standard Bluetooth SIG services & profiles
        "00001800-0000-1000-8000-00805f9b34fb" => ("Generic Access (GAP)", false),
        "00001801-0000-1000-8000-00805f9b34fb" => ("Generic Attribute (GATT)", false),
        "0000180a-0000-1000-8000-00805f9b34fb" => ("Device Information (DIS)", false),
        "0000180f-0000-1000-8000-00805f9b34fb" => ("Battery Service (BAS)", false),
        "00001812-0000-1000-8000-00805f9b34fb" => ("HID over GATT (HOGP)", false),
        "00001844-0000-1000-8000-00805f9b34fb" => ("Volume Control Service (VCS)", false),
        "0000184d-0000-1000-8000-00805f9b34fb" => ("Microphone Control Service (MICS)", false),
        "0000184e-0000-1000-8000-00805f9b34fb" => ("Audio Stream Control (ASCS)", false),
        "00001850-0000-1000-8000-00805f9b34fb" => ("Published Audio Capabilities (PACS)", false),
        "00001853-0000-1000-8000-00805f9b34fb" => ("Common Audio Service (CAS)", false),
        "0000110b-0000-1000-8000-00805f9b34fb" => ("A2DP Audio Sink", false),
        "0000110a-0000-1000-8000-00805f9b34fb" => ("A2DP Audio Source", false),
        "0000110c-0000-1000-8000-00805f9b34fb" => ("AVRCP Target", false),
        "0000110e-0000-1000-8000-00805f9b34fb" => ("AVRCP Controller", false),
        "0000111e-0000-1000-8000-00805f9b34fb" => ("Hands-Free (HFP)", false),
        "00001108-0000-1000-8000-00805f9b34fb" => ("Headset (HSP)", false),
        "00001124-0000-1000-8000-00805f9b34fb" => ("Human Interface Device (HID)", false),
        "00001101-0000-1000-8000-00805f9b34fb" => ("Serial Port Profile (SPP)", false),
        "0000fe2c-0000-1000-8000-00805f9b34fb" => ("Google Fast Pair", false),
        _ => {
            if lower.ends_with("-0000-1000-8000-00805f9b34fb") {
                ("Bluetooth SIG Assigned", false)
            } else {
                ("Vendor-Specific UUID", true)
            }
        }
    }
}

impl BtCmd {
    pub async fn run(self) -> Result<ExitCode> {
        #[cfg(not(target_os = "linux"))]
        {
            bail!("Bluetooth GATT inspection tools currently require Linux BlueZ D-Bus.");
        }

        #[cfg(target_os = "linux")]
        {
            match self {
                Self::List(args) => linux::run_list(args).await,
                Self::Gatt(args) => linux::run_gatt(args).await,
                Self::Read(args) => linux::run_read(args).await,
                Self::Write(args) => linux::run_write(args).await,
                Self::Watch(args) => linux::run_watch(args).await,
                Self::Snapshot(args) => linux::run_snapshot(args).await,
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::process::ExitCode;
    use std::time::Duration;

    use anyhow::{Context, Result, bail};
    use futures_lite::StreamExt as _;
    use zbus::Connection;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

    use super::{
        BtDeviceSnapshot, BtGattCharSnapshot, BtGattDescSnapshot, BtGattServiceSnapshot,
        BtUuidEntry, GattArgs, ListBtArgs, ReadArgs, SnapshotArgs, WatchArgs, WriteArgs,
        classify_uuid, format_hex_ascii, parse_hex_bytes, parse_modalias, redact_mac,
    };

    type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

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

    fn prop_str_array(props: &HashMap<String, OwnedValue>, key: &str) -> Vec<String> {
        props
            .get(key)
            .map(|v| match &**v {
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| match item {
                        Value::Str(s) => Some(s.to_string()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .unwrap_or_default()
    }

    fn prop_bytes(props: &HashMap<String, OwnedValue>, key: &str) -> Option<Vec<u8>> {
        let v = props.get(key)?;
        match &**v {
            Value::Array(arr) => {
                let bytes: Vec<u8> = arr
                    .iter()
                    .filter_map(|item| match item {
                        Value::U8(b) => Some(*b),
                        _ => None,
                    })
                    .collect();
                Some(bytes)
            }
            _ => None,
        }
    }

    async fn fetch_managed_objects(connection: &Connection) -> Result<ManagedObjects> {
        let reply = connection
            .call_method(
                Some("org.bluez"),
                "/",
                Some("org.freedesktop.DBus.ObjectManager"),
                "GetManagedObjects",
                &(),
            )
            .await
            .context("failed to call org.freedesktop.DBus.ObjectManager.GetManagedObjects on org.bluez")?;

        reply
            .body()
            .deserialize()
            .context("failed to deserialize BlueZ managed objects")
    }

    fn resolve_device_path(
        managed: &ManagedObjects,
        address: Option<&str>,
    ) -> Result<(OwnedObjectPath, String)> {
        let mut candidates: Vec<(OwnedObjectPath, String, bool)> = Vec::new();
        for (path, ifaces) in managed {
            let Some(props) = ifaces.get("org.bluez.Device1") else {
                continue;
            };
            let addr = prop_str(props, "Address").unwrap_or_default();
            let connected = prop_bool(props, "Connected");
            candidates.push((path.clone(), addr, connected));
        }

        if candidates.is_empty() {
            bail!("No Bluetooth devices found in BlueZ.");
        }

        if let Some(target) = address {
            let found = candidates
                .into_iter()
                .find(|(_, addr, _)| addr.eq_ignore_ascii_case(target))
                .with_context(|| format!("No BlueZ device found for MAC address {target}"))?;
            return Ok((found.0, found.1));
        }

        let connected: Vec<_> = candidates
            .iter()
            .filter(|(_, _, conn)| *conn)
            .cloned()
            .collect();
        match connected.len() {
            1 => {
                let first = connected.into_iter().next().context("missing device")?;
                Ok((first.0, first.1))
            }
            0 if candidates.len() == 1 => {
                let first = candidates.into_iter().next().context("missing device")?;
                Ok((first.0, first.1))
            }
            0 => bail!("No Bluetooth devices are currently connected. Specify --address."),
            _ => bail!("Multiple Bluetooth devices are connected. Specify --address."),
        }
    }

    async fn read_char_value(connection: &Connection, char_path: &str) -> Result<Vec<u8>> {
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let reply = connection
            .call_method(
                Some("org.bluez"),
                char_path,
                Some("org.bluez.GattCharacteristic1"),
                "ReadValue",
                &(options,),
            )
            .await
            .with_context(|| format!("GattCharacteristic1.ReadValue failed on {char_path}"))?;
        let bytes: Vec<u8> = reply.body().deserialize()?;
        Ok(bytes)
    }

    async fn collect_gatt_services(
        connection: &Connection,
        managed: &ManagedObjects,
        dev_prefix: &str,
        read_live: bool,
    ) -> Vec<BtGattServiceSnapshot> {
        let mut services: Vec<(String, BtGattServiceSnapshot)> = Vec::new();
        let mut chars: Vec<(String, String, BtGattCharSnapshot)> = Vec::new();
        let mut descs: Vec<(String, BtGattDescSnapshot)> = Vec::new();

        for (path, obj_ifaces) in managed {
            let p = path.as_str();
            if !p.starts_with(dev_prefix) {
                continue;
            }
            if let Some(svc_props) = obj_ifaces.get("org.bluez.GattService1") {
                let uuid = prop_str(svc_props, "UUID").unwrap_or_default();
                let (label, _) = classify_uuid(&uuid);
                services.push((
                    p.to_string(),
                    BtGattServiceSnapshot {
                        uuid,
                        label: label.to_string(),
                        primary: prop_bool(svc_props, "Primary"),
                        characteristics: Vec::new(),
                    },
                ));
            } else if let Some(char_props) = obj_ifaces.get("org.bluez.GattCharacteristic1") {
                let uuid = prop_str(char_props, "UUID").unwrap_or_default();
                let (label, _) = classify_uuid(&uuid);
                let flags = prop_str_array(char_props, "Flags");
                let svc_path = char_props
                    .get("Service")
                    .and_then(|v| match &**v {
                        Value::ObjectPath(op) => Some(op.as_str().to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let mut val = prop_bytes(char_props, "Value");
                if read_live
                    && flags.iter().any(|f| f == "read")
                    && let Ok(live) = read_char_value(connection, p).await
                {
                    val = Some(live);
                }
                let value_hex = val.map(|b| format_hex_ascii(&b));

                chars.push((
                    p.to_string(),
                    svc_path,
                    BtGattCharSnapshot {
                        uuid,
                        label: label.to_string(),
                        flags,
                        value_hex,
                        descriptors: Vec::new(),
                    },
                ));
            } else if let Some(desc_props) = obj_ifaces.get("org.bluez.GattDescriptor1") {
                let uuid = prop_str(desc_props, "UUID").unwrap_or_default();
                let char_path = desc_props
                    .get("Characteristic")
                    .and_then(|v| match &**v {
                        Value::ObjectPath(op) => Some(op.as_str().to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let value_hex = prop_bytes(desc_props, "Value").map(|b| format_hex_ascii(&b));
                descs.push((char_path, BtGattDescSnapshot { uuid, value_hex }));
            }
        }

        for (char_path, desc) in descs {
            if let Some((_, _, c)) = chars.iter_mut().find(|(p, _, _)| *p == char_path) {
                c.descriptors.push(desc);
            }
        }
        chars.sort_by(|a, b| a.0.cmp(&b.0));

        for (_, svc_path, c) in chars {
            if let Some((_, s)) = services.iter_mut().find(|(p, _)| *p == svc_path) {
                s.characteristics.push(c);
            }
        }
        services.sort_by(|a, b| a.0.cmp(&b.0));
        services.into_iter().map(|(_, s)| s).collect()
    }

    async fn build_device_snapshot(
        connection: &Connection,
        managed: &ManagedObjects,
        dev_path: &OwnedObjectPath,
        read_live: bool,
        redact: bool,
    ) -> Result<BtDeviceSnapshot> {
        let ifaces = managed
            .get(dev_path)
            .context("device path missing from BlueZ managed objects")?;
        let props = ifaces
            .get("org.bluez.Device1")
            .context("org.bluez.Device1 missing")?;

        let raw_addr = prop_str(props, "Address").unwrap_or_default();
        let address = if redact {
            redact_mac(&raw_addr)
        } else {
            raw_addr
        };
        let name = prop_str(props, "Name").unwrap_or_default();
        let alias = prop_str(props, "Alias").unwrap_or_else(|| name.clone());
        let connected = prop_bool(props, "Connected");
        let paired = prop_bool(props, "Paired");
        let trusted = prop_bool(props, "Trusted");
        let rssi = props
            .get("RSSI")
            .and_then(|v| v.downcast_ref::<i16>().ok());
        let battery_percentage = ifaces
            .get("org.bluez.Battery1")
            .and_then(|b| b.get("Percentage"))
            .and_then(|v| v.downcast_ref::<u8>().ok());
        let modalias = prop_str(props, "Modalias").and_then(|m| parse_modalias(&m));

        let uuids = prop_str_array(props, "UUIDs")
            .into_iter()
            .map(|uuid| {
                let (label, is_vendor) = classify_uuid(&uuid);
                BtUuidEntry {
                    uuid,
                    label: label.to_string(),
                    is_vendor,
                }
            })
            .collect();

        let services =
            collect_gatt_services(connection, managed, dev_path.as_str(), read_live).await;

        Ok(BtDeviceSnapshot {
            address,
            name,
            alias,
            connected,
            paired,
            trusted,
            rssi,
            battery_percentage,
            modalias,
            uuids,
            services,
        })
    }

    pub(super) async fn run_list(args: ListBtArgs) -> Result<ExitCode> {
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;

        let mut snapshots = Vec::new();
        for (path, ifaces) in &managed {
            if !ifaces.contains_key("org.bluez.Device1") {
                continue;
            }
            let snap = build_device_snapshot(&connection, &managed, path, false, false).await?;

            if let Some(ref filter) = args.filter {
                let f = filter.to_ascii_lowercase();
                let matches = snap.name.to_ascii_lowercase().contains(&f)
                    || snap.alias.to_ascii_lowercase().contains(&f)
                    || snap.address.to_ascii_lowercase().contains(&f);
                if !matches {
                    continue;
                }
            }

            if args.audio_only {
                let has_audio = snap.uuids.iter().any(|u| {
                    u.is_vendor
                        || u.label.contains("A2DP")
                        || u.label.contains("HFP")
                        || u.label.contains("HSP")
                        || u.label.contains("Audio")
                });
                if !has_audio {
                    continue;
                }
            }

            snapshots.push(snap);
        }

        snapshots.sort_by(|a, b| b.connected.cmp(&a.connected).then_with(|| a.address.cmp(&b.address)));

        if args.json {
            println!("{}", serde_json::to_string_pretty(&snapshots)?);
            return Ok(ExitCode::SUCCESS);
        }

        if snapshots.is_empty() {
            println!("No matching Bluetooth devices found in BlueZ.");
            return Ok(ExitCode::from(2));
        }

        for snap in snapshots {
            let state = if snap.connected { "Connected" } else { "Offline" };
            let paired = if snap.paired { "Paired" } else { "Unpaired" };
            let bat = snap
                .battery_percentage
                .map_or_else(|| "N/A".to_string(), |b| format!("{b}%"));
            let vid_pid = snap.modalias.as_ref().map_or_else(
                || "----:----".to_string(),
                |m| format!("{:04x}:{:04x}", m.vendor_id, m.product_id),
            );
            let vendor_uuids: Vec<&str> = snap
                .uuids
                .iter()
                .filter(|u| u.is_vendor)
                .map(|u| u.uuid.as_str())
                .collect();

            println!(
                "{}  [{}, {}]  {}  VID:PID {}  Bat: {}  GATT svcs: {}",
                snap.address,
                state,
                paired,
                if snap.alias.is_empty() {
                    &snap.name
                } else {
                    &snap.alias
                },
                vid_pid,
                bat,
                snap.services.len()
            );
            if !vendor_uuids.is_empty() {
                println!("    Vendor UUIDs: {}", vendor_uuids.join(", "));
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    pub(super) async fn run_gatt(args: GattArgs) -> Result<ExitCode> {
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;
        let (dev_path, _) = resolve_device_path(&managed, args.address.as_deref())?;
        let snap = build_device_snapshot(&connection, &managed, &dev_path, args.read, false).await?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&snap)?);
            return Ok(ExitCode::SUCCESS);
        }

        println!("{} ({}) — {}", snap.alias, snap.address, if snap.connected { "Connected" } else { "Offline" });
        if snap.services.is_empty() {
            println!("  No GATT services resolved (device may be offline or Classic-only).");
            return Ok(ExitCode::SUCCESS);
        }

        for svc in &snap.services {
            let kind = if svc.primary { "primary" } else { "secondary" };
            println!("  Service {} [{}] ({})", svc.uuid, kind, svc.label);
            for ch in &svc.characteristics {
                println!(
                    "    Char  {} [{}] ({})",
                    ch.uuid,
                    ch.flags.join(", "),
                    ch.label
                );
                if let Some(ref val) = ch.value_hex {
                    println!("      Value: {val}");
                }
                for desc in &ch.descriptors {
                    println!("      Desc  {}  {}", desc.uuid, desc.value_hex.as_deref().unwrap_or(""));
                }
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn find_char_path(
        managed: &ManagedObjects,
        dev_path: &OwnedObjectPath,
        char_uuid_query: &str,
    ) -> Result<OwnedObjectPath> {
        let dev_prefix = dev_path.as_str();
        let query = char_uuid_query.to_ascii_lowercase();
        let mut matches = Vec::new();
        for (path, ifaces) in managed {
            if !path.as_str().starts_with(dev_prefix) {
                continue;
            }
            let Some(props) = ifaces.get("org.bluez.GattCharacteristic1") else {
                continue;
            };
            let Some(uuid) = prop_str(props, "UUID") else {
                continue;
            };
            if uuid.eq_ignore_ascii_case(&query) || uuid.to_ascii_lowercase().starts_with(&query) {
                matches.push((path.clone(), uuid));
            }
        }

        match matches.len() {
            0 => bail!("No GATT characteristic matching '{char_uuid_query}' found on {dev_prefix}"),
            1 => Ok(matches.into_iter().next().context("missing char")?.0),
            _ => bail!("Ambiguous characteristic UUID prefix '{char_uuid_query}' matched {} characteristics", matches.len()),
        }
    }

    pub(super) async fn run_read(args: ReadArgs) -> Result<ExitCode> {
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;
        let (dev_path, addr) = resolve_device_path(&managed, args.address.as_deref())?;
        let char_path = find_char_path(&managed, &dev_path, &args.char_uuid)?;
        let bytes = read_char_value(&connection, char_path.as_str()).await?;
        println!("{} [{}]: {}", addr, args.char_uuid, format_hex_ascii(&bytes));
        Ok(ExitCode::SUCCESS)
    }

    pub(super) async fn run_write(args: WriteArgs) -> Result<ExitCode> {
        let payload = parse_hex_bytes(&args.hex_bytes)?;
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;
        let (dev_path, addr) = resolve_device_path(&managed, args.address.as_deref())?;
        let char_path = find_char_path(&managed, &dev_path, &args.char_uuid)?;

        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert(
            "type",
            Value::from(if args.with_response { "request" } else { "command" }),
        );

        connection
            .call_method(
                Some("org.bluez"),
                char_path.as_str(),
                Some("org.bluez.GattCharacteristic1"),
                "WriteValue",
                &(payload.as_slice(), options),
            )
            .await
            .with_context(|| format!("GattCharacteristic1.WriteValue failed on {}", char_path.as_str()))?;

        println!(
            "Wrote {} bytes to {} [{}]: {}",
            payload.len(),
            addr,
            args.char_uuid,
            format_hex_ascii(&payload)
        );
        Ok(ExitCode::SUCCESS)
    }

    pub(super) async fn run_watch(args: WatchArgs) -> Result<ExitCode> {
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;
        let (dev_path, addr) = resolve_device_path(&managed, args.address.as_deref())?;

        let mut target_chars: Vec<(OwnedObjectPath, String)> = Vec::new();
        if let Some(ref uuid_q) = args.char_uuid {
            let p = find_char_path(&managed, &dev_path, uuid_q)?;
            target_chars.push((p, uuid_q.clone()));
        } else {
            let dev_prefix = dev_path.as_str();
            for (path, ifaces) in &managed {
                if !path.as_str().starts_with(dev_prefix) {
                    continue;
                }
                let Some(props) = ifaces.get("org.bluez.GattCharacteristic1") else {
                    continue;
                };
                let flags = prop_str_array(props, "Flags");
                if flags.iter().any(|f| f == "notify" || f == "indicate") {
                    let uuid = prop_str(props, "UUID").unwrap_or_default();
                    target_chars.push((path.clone(), uuid));
                }
            }
        }

        if target_chars.is_empty() {
            bail!("No notify/indicate GATT characteristics found on {addr}.");
        }

        let rule = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.bluez")?
            .interface("org.freedesktop.DBus.Properties")?
            .member("PropertiesChanged")?
            .build();

        let mut stream = zbus::MessageStream::for_match_rule(rule, &connection, Some(64)).await?;

        for (char_path, uuid) in &target_chars {
            if let Err(err) = connection
                .call_method(
                    Some("org.bluez"),
                    char_path.as_str(),
                    Some("org.bluez.GattCharacteristic1"),
                    "StartNotify",
                    &(),
                )
                .await
            {
                eprintln!("warn: StartNotify failed on {uuid}: {err}");
            } else {
                println!("Subscribed to {uuid} ({})", char_path.as_str());
            }
        }

        println!(
            "Watching {} characteristic(s) on {addr} ({}s, Ctrl-C to stop)...",
            target_chars.len(),
            args.duration
        );

        let watch_loop = async {
            while let Some(Ok(msg)) = stream.next().await {
                let Some(obj_path) = msg.header().path().map(|p| p.as_str().to_string()) else {
                    continue;
                };
                let Some((_, uuid)) = target_chars
                    .iter()
                    .find(|(p, _)| p.as_str() == obj_path)
                else {
                    continue;
                };
                if let Ok((iface, changed, _)) = msg.body().deserialize::<(
                    String,
                    HashMap<String, OwnedValue>,
                    Vec<String>,
                )>() && iface == "org.bluez.GattCharacteristic1"
                    && let Some(bytes) = prop_bytes(&changed, "Value")
                {
                    println!("[{uuid}] {}", format_hex_ascii(&bytes));
                }
            }
        };

        if args.duration > 0 {
            let _ = tokio::time::timeout(Duration::from_secs(args.duration), watch_loop).await;
        } else {
            watch_loop.await;
        }

        for (char_path, _) in &target_chars {
            let _ = connection
                .call_method(
                    Some("org.bluez"),
                    char_path.as_str(),
                    Some("org.bluez.GattCharacteristic1"),
                    "StopNotify",
                    &(),
                )
                .await;
        }

        Ok(ExitCode::SUCCESS)
    }

    pub(super) async fn run_snapshot(args: SnapshotArgs) -> Result<ExitCode> {
        let connection = Connection::system().await?;
        let managed = fetch_managed_objects(&connection).await?;
        let (dev_path, _) = resolve_device_path(&managed, args.address.as_deref())?;
        let snap = build_device_snapshot(&connection, &managed, &dev_path, args.read, args.redact_mac).await?;
        let json = serde_json::to_string_pretty(&snap)?;

        if let Some(out) = args.out {
            std::fs::write(&out, format!("{json}\n"))
                .with_context(|| format!("failed to write snapshot to {}", out.display()))?;
            println!("Wrote Bluetooth snapshot to {}", out.display());
        } else {
            println!("{json}");
        }

        Ok(ExitCode::SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flexible_hex_formats() {
        assert_eq!(parse_hex_bytes("c30003ff0000").expect("contiguous"), vec![0xc3, 0x00, 0x03, 0xff, 0x00, 0x00]);
        assert_eq!(parse_hex_bytes("c3 00 03 ff 00 00").expect("spaced"), vec![0xc3, 0x00, 0x03, 0xff, 0x00, 0x00]);
        assert_eq!(parse_hex_bytes("0xc3,0x00,0x01").expect("prefixed"), vec![0xc3, 0x00, 0x01]);
        let _ = parse_hex_bytes("").unwrap_err();
        let _ = parse_hex_bytes("abc").unwrap_err();
        let _ = parse_hex_bytes("zz").unwrap_err();
    }

    #[test]
    fn formats_hex_and_ascii() {
        assert_eq!(format_hex_ascii(&[0x41, 0x42, 0x00]), "41 42 00  |AB.|");
        assert_eq!(format_hex_ascii(&[]), "<empty>");
    }

    #[test]
    fn parses_modalias_and_redacts_mac() {
        let parsed = parse_modalias("bluetooth:v1532p0544d0100").expect("modalias");
        assert_eq!(parsed.source, "bluetooth");
        assert_eq!(parsed.vendor_id, 0x1532);
        assert_eq!(parsed.product_id, 0x0544);
        assert_eq!(parsed.version, Some(0x0100));

        assert_eq!(redact_mac("44:5E:CD:11:C4:66"), "44:5E:CD:00:00:00");
    }

    #[test]
    fn classifies_known_and_vendor_uuids() {
        assert_eq!(
            classify_uuid("416d656c-2d52-617a-6572-424c4501f40a"),
            ("Razer BLE Audio Service", true)
        );
        assert_eq!(
            classify_uuid("0000110b-0000-1000-8000-00805f9b34fb"),
            ("A2DP Audio Sink", false)
        );
        assert_eq!(
            classify_uuid("12345678-1234-1234-1234-123456789abc"),
            ("Vendor-Specific UUID", true)
        );
    }
}

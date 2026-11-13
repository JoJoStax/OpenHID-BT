//! Device representation and integration into OpenHID/BT.

use openlogi_core::device::{DeviceKind, RawDeviceAddress, StandaloneDevice};
use serde::{Deserialize, Serialize};

use crate::protocol::commands::{AutoStandby, RazerLedMode};
use crate::protocol::uuids::RAZER_VENDOR_ID;
use crate::transport::DiscoveredRazerDevice;

/// Stable driver-family identifier for Razer Bluetooth audio devices.
pub const RAZER_DRIVER_ID: &str = "razer";

/// High-level state of a Razer Bluetooth headset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RazerDeviceState {
    /// Bluetooth MAC address.
    pub address: String,
    /// Advertised device name.
    pub name: String,
    /// Whether the headset is currently connected.
    pub connected: bool,
    /// Battery percentage, if reported.
    pub battery_percentage: Option<u8>,
    /// Last applied Chroma BLE LED mode.
    pub led_mode: Option<RazerLedMode>,
    /// Last applied auto-standby timeout.
    pub auto_standby: Option<AutoStandby>,
}

/// Convert a discovered BlueZ Razer device into OpenHID/BT's [`StandaloneDevice`].
#[must_use]
pub fn standalone_device_from_razer(device: &DiscoveredRazerDevice) -> StandaloneDevice {
    let display_name = if device.model.model_id == "razer-bt-generic" && !device.alias.is_empty() {
        device.alias.clone()
    } else {
        device.model.marketing_name.to_string()
    };
    let identity = format!("bt:{}", device.address);

    StandaloneDevice {
        address: RawDeviceAddress {
            vendor_id: RAZER_VENDOR_ID,
            product_id: device.model.product_ids.first().copied().unwrap_or(0),
            usage_page: 0,
            usage_id: 0,
            identity,
        },
        display_name,
        manufacturer: Some("Razer".to_string()),
        serial_number: Some(device.address.clone()),
        unit_id: [0; 4],
        kind: DeviceKind::Headset,
        online: device.connected,
        capabilities: None,
        light_capabilities: None,
        driver_id: RAZER_DRIVER_ID.to_string(),
        registry_model_id: Some(device.model.model_id.to_string()),
    }
}

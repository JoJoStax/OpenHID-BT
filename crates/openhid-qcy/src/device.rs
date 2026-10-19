//! Device representation and integration into OpenHID/BT.

use openlogi_core::device::{DeviceKind, RawDeviceAddress, StandaloneDevice};
use serde::{Deserialize, Serialize};

use crate::protocol::commands::AncMode;
use crate::protocol::uuids::HELE_QCY_VENDOR_ID;
use crate::transport::DiscoveredQcyDevice;

pub const QCY_DRIVER_ID: &str = "qcy";

/// High-level state of a QCY Bluetooth device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcyDeviceState {
    pub address: String,
    pub name: String,
    pub connected: bool,
    pub battery_percentage: Option<u8>,
    pub anc_mode: Option<AncMode>,
    pub game_mode: Option<bool>,
}

/// Convert a discovered BlueZ QCY device into OpenHID/BT's `StandaloneDevice`.
#[must_use]
pub fn standalone_device_from_qcy(device: &DiscoveredQcyDevice) -> StandaloneDevice {
    let display_name = device.model.marketing_name.to_string();
    let identity = format!("bt:{}", device.address);

    StandaloneDevice {
        address: RawDeviceAddress {
            vendor_id: HELE_QCY_VENDOR_ID,
            product_id: device.model.product_ids.first().copied().unwrap_or(0),
            usage_page: 0,
            usage_id: 0,
            identity,
        },
        display_name,
        manufacturer: Some("QCY".to_string()),
        serial_number: Some(device.address.clone()),
        unit_id: [0; 4],
        kind: DeviceKind::Headset,
        online: device.connected,
        capabilities: None,
        light_capabilities: None,
        driver_id: QCY_DRIVER_ID.to_string(),
        registry_model_id: Some(device.model.model_id.to_string()),
    }
}

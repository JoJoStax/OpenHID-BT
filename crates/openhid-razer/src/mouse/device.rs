//! StandaloneDevice conversion for Razer USB/2.4GHz mice.

use openlogi_core::device::{Capabilities, DeviceKind, RawDeviceAddress, StandaloneDevice};

use super::catalog::RAZER_USB_VENDOR_ID;
use super::transport::DiscoveredRazerMouse;

/// Stable driver-family identifier for Razer USB/2.4GHz mice.
pub const RAZER_MOUSE_DRIVER_ID: &str = "razer-mouse";

/// Convert a discovered Razer USB/2.4GHz mouse into OpenHID/BT's [`StandaloneDevice`].
#[must_use]
pub fn standalone_device_from_razer_mouse(mouse: &DiscoveredRazerMouse) -> StandaloneDevice {
    let display_name = if mouse.model.model_id == "razer-mouse-generic"
        && !mouse.os_name.trim().is_empty()
    {
        mouse.os_name.clone()
    } else {
        mouse.model.marketing_name.to_string()
    };

    let identity = mouse
        .serial_number
        .as_deref()
        .filter(|s| !s.is_empty())
        .map_or_else(
            || format!("razer-usb:{:04x}", mouse.product_id),
            |s| format!("serial:{}", s.to_ascii_lowercase()),
        );

    let caps = Capabilities {
        pointer: true,
        lighting: mouse.model.capabilities.has_chroma,
        ..Capabilities::default()
    };

    StandaloneDevice {
        address: RawDeviceAddress {
            vendor_id: RAZER_USB_VENDOR_ID,
            product_id: mouse.product_id,
            usage_page: mouse.usage_page,
            usage_id: mouse.usage_id,
            identity,
        },
        display_name,
        manufacturer: Some("Razer".to_string()),
        serial_number: mouse.serial_number.clone(),
        unit_id: [0; 4],
        kind: DeviceKind::Mouse,
        online: true,
        capabilities: Some(caps),
        light_capabilities: None,
        driver_id: RAZER_MOUSE_DRIVER_ID.to_string(),
        registry_model_id: Some(mouse.model.model_id.to_string()),
    }
}

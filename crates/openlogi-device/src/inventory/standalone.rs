//! Discovery of standalone raw-HID devices.

use std::collections::{HashMap, HashSet};

use openlogi_core::device::{DeviceKind, RawDeviceAddress, StandaloneDevice};
use openlogi_device_registry::headset::{GAMING_HEADSET_DRIVER_ID, find_gaming_headset};
use openlogi_device_registry::litra::find_litra;

use super::InventoryError;
use crate::backend::HidBackend;
use crate::write::litra_capabilities;

/// Enumerate recognized standalone devices without probing them as HID++.
///
/// The returned descriptors are intentionally separate from receiver
/// inventories. A raw device has no HID++ pairing slot and must be routed by
/// its full HID identity tuple.
pub async fn enumerate_standalone(
    backend: &dyn HidBackend,
) -> Result<Vec<StandaloneDevice>, InventoryError> {
    let devices = backend.enumerate().await?;
    let devices: Vec<_> = devices
        .into_iter()
        .filter_map(|device| standalone_device(&device))
        .collect();
    validate_no_ambiguous_nodes(&devices)?;
    Ok(devices)
}

/// Classifier function that inspects a raw HID [`crate::backend::NodeInfo`] and returns a
/// [`StandaloneDevice`] if it belongs to that driver family.
pub type StandaloneClassifier = fn(&crate::backend::NodeInfo) -> Option<StandaloneDevice>;

/// Registered standalone HID/BT device classifiers, evaluated in precedence order.
///
/// New peripheral driver families (e.g. third-party HID/Bluetooth mice, keyboards,
/// audio devices, or lighting controllers) can be registered here without altering
/// HID++ receiver or direct-device discovery.
const STANDALONE_CLASSIFIERS: &[StandaloneClassifier] = &[classify_litra, classify_gaming_headset];

/// Classifies one enumerated HID node against every standalone driver family
/// OpenHID/BT recognizes, in precedence order. `None` when no family claims it.
fn standalone_device(device: &crate::backend::NodeInfo) -> Option<StandaloneDevice> {
    STANDALONE_CLASSIFIERS
        .iter()
        .find_map(|classify| classify(device))
}

fn classify_litra(device: &crate::backend::NodeInfo) -> Option<StandaloneDevice> {
    let descriptor = find_litra(
        device.vendor_id,
        device.product_id,
        device.usage_page,
        device.usage_id,
    )?;
    Some(StandaloneDevice {
        address: RawDeviceAddress {
            vendor_id: device.vendor_id,
            product_id: device.product_id,
            usage_page: device.usage_page,
            usage_id: device.usage_id,
            identity: device.identity(),
        },
        display_name: device.name.clone(),
        manufacturer: device.manufacturer.clone(),
        serial_number: device.serial_number.clone(),
        unit_id: [0; 4],
        kind: DeviceKind::Light,
        online: true,
        capabilities: None,
        light_capabilities: Some(litra_capabilities(descriptor.model)),
        driver_id: descriptor.driver_id.to_owned(),
        registry_model_id: Some(descriptor.registry_model_id.to_owned()),
    })
}

fn classify_gaming_headset(device: &crate::backend::NodeInfo) -> Option<StandaloneDevice> {
    let descriptor = find_gaming_headset(
        device.vendor_id,
        device.product_id,
        device.usage_page,
        device.usage_id,
    )?;
    Some(StandaloneDevice {
        address: RawDeviceAddress {
            vendor_id: device.vendor_id,
            product_id: device.product_id,
            usage_page: device.usage_page,
            usage_id: device.usage_id,
            identity: device.identity(),
        },
        display_name: descriptor.name.to_owned(),
        manufacturer: device.manufacturer.clone(),
        serial_number: device.serial_number.clone(),
        unit_id: [0; 4],
        kind: DeviceKind::Headset,
        online: true,
        capabilities: None,
        light_capabilities: None,
        driver_id: GAMING_HEADSET_DRIVER_ID.to_owned(),
        registry_model_id: Some(descriptor.registry_model_id.to_owned()),
    })
}

/// Reject multiple nodes that the route cannot distinguish safely.
///
/// A serial-bearing pair is distinguishable even when two identical lights
/// share the same VID/PID/usage tuple. An OS-node identity (`id:…`) is only a
/// transient re-find hint, so two such nodes with the same tuple are
/// indistinguishable and must not be exposed as independently selectable
/// devices.
fn validate_no_ambiguous_nodes(devices: &[StandaloneDevice]) -> Result<(), InventoryError> {
    if devices.len() < 2 {
        return Ok(());
    }
    let mut groups: HashMap<(u16, u16, u16, u16), Vec<&StandaloneDevice>> =
        HashMap::with_capacity(devices.len());
    for device in devices {
        let address = &device.address;
        groups
            .entry((
                address.vendor_id,
                address.product_id,
                address.usage_page,
                address.usage_id,
            ))
            .or_default()
            .push(device);
    }
    if groups.values().any(|group| {
        if group.len() < 2 {
            return false;
        }
        let identities: HashSet<&str> = group
            .iter()
            .map(|device| device.address.identity.as_str())
            .collect();
        identities.len() != group.len()
            || group
                .iter()
                .any(|device| !device.address.identity.starts_with("serial:"))
    }) {
        return Err(InventoryError::AmbiguousRawDevice);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use openlogi_core::device::{DeviceKind, RawDeviceAddress, StandaloneDevice};

    use crate::write::matches_litra;

    use super::{InventoryError, validate_no_ambiguous_nodes};

    #[test]
    fn glow_fixture_matches_standalone_driver() {
        assert!(matches_litra(0x046d, 0xc900, 0xff43, 0x0202));
    }

    fn raw(identity: &str) -> StandaloneDevice {
        StandaloneDevice {
            address: RawDeviceAddress {
                vendor_id: 0x046d,
                product_id: 0xc900,
                usage_page: 0xff43,
                usage_id: 0x0202,
                identity: identity.into(),
            },
            display_name: "Litra Glow".into(),
            manufacturer: Some("Logi".into()),
            serial_number: identity.strip_prefix("serial:").map(str::to_string),
            unit_id: [0; 4],
            kind: DeviceKind::Light,
            online: true,
            capabilities: None,
            light_capabilities: None,
            driver_id: "litra".into(),
            registry_model_id: None,
        }
    }

    #[test]
    fn duplicate_transient_nodes_are_rejected() {
        let devices = vec![raw("id:old"), raw("id:new")];
        assert!(matches!(
            validate_no_ambiguous_nodes(&devices),
            Err(InventoryError::AmbiguousRawDevice)
        ));
    }

    #[test]
    fn distinct_serials_can_share_the_same_hid_tuple() {
        let devices = vec![raw("serial:one"), raw("serial:two")];
        let validation = validate_no_ambiguous_nodes(&devices);
        assert!(
            validation.is_ok(),
            "serial-backed nodes stay distinguishable: {validation:?}"
        );
    }
}

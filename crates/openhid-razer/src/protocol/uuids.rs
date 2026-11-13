//! Well-known Bluetooth UUIDs, vendor IDs, and OUI prefixes for Razer devices.

/// Razer Inc. USB / Bluetooth SIG Vendor ID (`0x1532`).
pub const RAZER_VENDOR_ID: u16 = 0x1532;

/// Razer BLE GATT primary service UUID (`"Amel-RazerBLE\x01\xf4\x0a"`).
pub const RAZER_BLE_SERVICE_UUID: &str = "416d656c-2d52-617a-6572-424c4501f40a";

/// Razer BLE GATT command characteristic UUID (`WRITE NO RESPONSE`).
pub const RAZER_BLE_COMMAND_UUID: &str = "416d0000-2d52-617a-6572-424c4501f40a";

/// Known IEEE OUI prefixes assigned to Razer Bluetooth audio devices.
pub const RAZER_BT_OUIS: &[&str] = &["44:5E:CD", "00:14:41"];

/// Check whether a USB/Bluetooth vendor ID belongs to Razer Inc.
#[must_use]
pub const fn is_razer_vendor(vendor_id: u16) -> bool {
    vendor_id == RAZER_VENDOR_ID
}

/// Check whether a Bluetooth MAC address matches a known Razer audio OUI prefix.
#[must_use]
pub fn is_razer_oui(address: &str) -> bool {
    RAZER_BT_OUIS
        .iter()
        .any(|oui| address.len() >= oui.len() && address[..oui.len()].eq_ignore_ascii_case(oui))
}

/// Check whether a Bluetooth device name or alias identifies a Razer audio product.
#[must_use]
pub fn is_razer_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("razer")
        || lower.contains("kraken bt")
        || lower.contains("barracuda")
        || lower.contains("hammerhead")
        || lower.contains("blackshark")
        || lower.contains("kaira")
        || (lower.contains("opus") && !lower.contains("logitech"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_and_oui_matching() {
        assert!(is_razer_vendor(0x1532));
        assert!(!is_razer_vendor(0x046d));
        assert!(is_razer_oui("44:5E:CD:11:C4:66"));
        assert!(is_razer_oui("44:5e:cd:aa:bb:cc"));
        assert!(is_razer_oui("00:14:41:12:34:56"));
        assert!(!is_razer_oui("AA:BB:CC:11:22:33"));
    }

    #[test]
    fn name_matching() {
        assert!(is_razer_name("Razer Kraken BT Kitty Edition"));
        assert!(is_razer_name("Razer Barracuda X"));
        assert!(is_razer_name("Hammerhead Pro HyperSpeed"));
        assert!(!is_razer_name("QCY H3S"));
        assert!(!is_razer_name("Logitech G735"));
    }
}

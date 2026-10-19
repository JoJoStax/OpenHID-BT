//! Parser for QCY BLE Manufacturer Data advertisements (`0x521C`).

use serde::{Deserialize, Serialize};

/// Parsed QCY manufacturer data advertising payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QcyAdvertisement {
    pub vendor_id: u16,
    pub left_battery: u8,
    pub right_battery: u8,
    pub case_battery: u8,
    pub left_charging: bool,
    pub right_charging: bool,
    pub case_charging: bool,
    pub control_mac: Option<String>,
}

impl QcyAdvertisement {
    /// Parse QCY manufacturer data payload bytes (associated with company ID `0x521C`).
    #[must_use]
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let vendor_id = u16::from_be_bytes([data[0], data[1]]);
        let left = data[5];
        let right = data[6];
        let case = data[7];

        let control_mac = if data.len() >= 17 {
            Some(format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                data[12], data[11], data[13], data[16], data[15], data[14]
            ))
        } else {
            None
        };

        Some(Self {
            vendor_id,
            left_battery: (left & 0x7F).min(100),
            right_battery: (right & 0x7F).min(100),
            case_battery: (case & 0x7F).min(100),
            left_charging: (left & 0x80) != 0,
            right_charging: (right & 0x80) != 0,
            case_charging: (case & 0x80) != 0,
            control_mac,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_advertisement() {
        let mut data = vec![0; 20];
        data[0] = 0x5D;
        data[1] = 0x45; // vendor_id = 23877 (N70)
        data[5] = 80;   // Left 80%
        data[6] = 85 | 0x80; // Right 85%, charging
        data[7] = 90;   // Case 90%
        data[11] = 0x22;
        data[12] = 0x11;
        data[13] = 0x33;
        data[14] = 0x66;
        data[15] = 0x55;
        data[16] = 0x44;

        let parsed = QcyAdvertisement::parse(&data).expect("parses");
        assert_eq!(parsed.vendor_id, 23877);
        assert_eq!(parsed.left_battery, 80);
        assert!(!parsed.left_charging);
        assert_eq!(parsed.right_battery, 85);
        assert!(parsed.right_charging);
        assert_eq!(parsed.case_battery, 90);
        assert_eq!(parsed.control_mac.as_deref(), Some("11:22:33:44:55:66"));
    }
}

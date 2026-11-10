//! Well-known Bluetooth UUIDs and identifiers for QCY devices.

/// QCY BLE GATT Main Service UUID.
pub const QCY_SERVICE_UUID: &str = "0000a001-0000-1000-8000-00805f9b34fb";

/// QCY BLE Command characteristic (write without response or write with response).
pub const QCY_COMMAND_UUID: &str = "00001001-0000-1000-8000-00805f9b34fb";

/// QCY BLE Notification characteristic (read, notify).
pub const QCY_NOTIFICATION_UUID: &str = "00001002-0000-1000-8000-00805f9b34fb";

/// QCY BLE Battery characteristic (read, notify).
pub const QCY_BATTERY_UUID: &str = "00000008-0000-1000-8000-00805f9b34fb";

/// QCY BLE Firmware Version characteristic (read).
pub const QCY_FIRMWARE_UUID: &str = "00000007-0000-1000-8000-00805f9b34fb";

/// QCY BLE Equalizer characteristic (legacy).
pub const QCY_EQUALIZER_UUID: &str = "0000000b-0000-1000-8000-00805f9b34fb";

/// QCY BLE Key Functions characteristic (read, write).
pub const QCY_KEY_FUNCTIONS_UUID: &str = "0000000d-0000-1000-8000-00805f9b34fb";

/// QCY Bluetooth SIG Manufacturer Company ID (`0x521C`).
pub const QCY_COMPANY_ID: u16 = 0x521C;

/// Dongguan Hele Electronics (QCY) Bluetooth SIG Vendor ID (`0x05D6` / `1494`).
pub const HELE_QCY_VENDOR_ID: u16 = 0x05D6;

/// Check whether a Bluetooth vendor ID or modalias belongs to QCY.
#[must_use]
pub fn is_qcy_vendor(vendor_id: u16) -> bool {
    vendor_id == HELE_QCY_VENDOR_ID || vendor_id == QCY_COMPANY_ID
}

/// Check whether a device name / alias is a QCY product.
#[must_use]
pub fn is_qcy_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("qcy") || lower.contains("melobuds") || lower.contains("arcbuds")
}

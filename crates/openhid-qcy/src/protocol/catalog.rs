//! Product catalog and model identifier mappings for QCY earbuds and headphones.

/// Known QCY models by vendor ID and marketed name.
#[must_use]
pub fn model_name_for_vendor(vendor_id: u16) -> &'static str {
    match vendor_id {
        23872 | 23877 => "QCY MeloBuds N70",
        19786 => "QCY MeloBuds Pro",
        23155 => "QCY MeloBuds Neo",
        21020 => "QCY MeloBuds",
        _ => "QCY Bluetooth Headset",
    }
}

/// Normalize or detect product model from Bluetooth device name or modalias.
#[must_use]
pub fn detect_product_name(raw_name: &str, vendor_id: Option<u16>) -> String {
    if let Some(vid) = vendor_id {
        if vid != 0 {
            return model_name_for_vendor(vid).to_string();
        }
    }

    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        "QCY Headset".to_string()
    } else {
        trimmed.to_string()
    }
}

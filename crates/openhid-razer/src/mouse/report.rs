//! 90-byte Razer USB/2.4GHz HID feature report framing and command builders.
//!
//! Matches the standard Razer HID protocol used across DeathAdder, Viper,
//! Basilisk, Naga, Cobra, Orochi, and HyperPolling dongles (as documented by
//! OpenRazer).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fixed length of a Razer HID feature report payload (excluding the leading
/// HID report ID `0x00` byte).
pub const RAZER_REPORT_LEN: usize = 90;

/// Maximum number of argument bytes inside a 90-byte Razer report.
pub const RAZER_MAX_ARGS: usize = 80;

/// Status byte in a 90-byte Razer HID report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RazerReportStatus {
    NewCommand = 0x00,
    Busy = 0x01,
    Successful = 0x02,
    Failure = 0x03,
    NoResponse = 0x04,
    NotSupported = 0x05,
}

impl RazerReportStatus {
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::NewCommand),
            0x01 => Some(Self::Busy),
            0x02 => Some(Self::Successful),
            0x03 => Some(Self::Failure),
            0x04 => Some(Self::NoResponse),
            0x05 => Some(Self::NotSupported),
            _ => None,
        }
    }
}

/// Errors returned by Razer mouse report encoding/decoding.
#[derive(Debug, Error)]
pub enum RazerMouseProtocolError {
    #[error("argument length {0} exceeds maximum {RAZER_MAX_ARGS}")]
    ArgsTooLong(usize),
    #[error("invalid Razer report length {0} (expected {RAZER_REPORT_LEN})")]
    InvalidReportLength(usize),
    #[error("invalid Razer report CRC: expected 0x{expected:02x}, got 0x{actual:02x}")]
    InvalidCrc { expected: u8, actual: u8 },
    #[error("unsupported polling rate {0} Hz")]
    UnsupportedPollingRate(u16),
    #[error("device reported failure status: {0:?}")]
    DeviceStatus(Option<RazerReportStatus>),
    #[error("DPI out of range ({0}); expected 50..=35000")]
    InvalidDpi(u16),
}

/// Standard 90-byte Razer HID feature report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RazerReport {
    pub status: u8,
    pub transaction_id: u8,
    pub remaining_packets: u16,
    pub protocol_type: u8,
    pub data_size: u8,
    pub command_class: u8,
    pub command_id: u8,
    pub arguments: [u8; RAZER_MAX_ARGS],
}

impl RazerReport {
    /// Create a new command report with `status = 0x00` (`NewCommand`).
    #[must_use]
    pub const fn new(transaction_id: u8, command_class: u8, command_id: u8, data_size: u8) -> Self {
        Self {
            status: RazerReportStatus::NewCommand as u8,
            transaction_id,
            remaining_packets: 0,
            protocol_type: 0x00,
            data_size,
            command_class,
            command_id,
            arguments: [0u8; RAZER_MAX_ARGS],
        }
    }

    /// Populate argument bytes from a slice (up to [`RAZER_MAX_ARGS`]).
    pub fn with_args(mut self, args: &[u8]) -> Result<Self, RazerMouseProtocolError> {
        if args.len() > RAZER_MAX_ARGS {
            return Err(RazerMouseProtocolError::ArgsTooLong(args.len()));
        }
        self.arguments[..args.len()].copy_from_slice(args);
        Ok(self)
    }

    /// Encode into a 90-byte Razer report with CRC in byte 88.
    #[must_use]
    pub fn encode(&self) -> [u8; RAZER_REPORT_LEN] {
        let mut buf = [0u8; RAZER_REPORT_LEN];
        buf[0] = self.status;
        buf[1] = self.transaction_id;
        let rem = self.remaining_packets.to_be_bytes();
        buf[2] = rem[0];
        buf[3] = rem[1];
        buf[4] = self.protocol_type;
        buf[5] = self.data_size;
        buf[6] = self.command_class;
        buf[7] = self.command_id;
        buf[8..88].copy_from_slice(&self.arguments);
        buf[88] = compute_crc(&buf);
        buf[89] = 0x00;
        buf
    }

    /// Encode with a leading `0x00` HID report ID byte (91 bytes total) for
    /// `hidraw` / `HidD_SetFeature`.
    #[must_use]
    pub fn encode_with_report_id(&self) -> [u8; RAZER_REPORT_LEN + 1] {
        let mut buf = [0u8; RAZER_REPORT_LEN + 1];
        buf[0] = 0x00;
        buf[1..].copy_from_slice(&self.encode());
        buf
    }

    /// Decode a 90-byte Razer report and verify its CRC.
    pub fn decode(raw: &[u8]) -> Result<Self, RazerMouseProtocolError> {
        let slice = if raw.len() == RAZER_REPORT_LEN + 1 && raw[0] == 0x00 {
            &raw[1..]
        } else {
            raw
        };
        if slice.len() != RAZER_REPORT_LEN {
            return Err(RazerMouseProtocolError::InvalidReportLength(raw.len()));
        }
        let expected = compute_crc(slice);
        let actual = slice[88];
        if expected != actual {
            return Err(RazerMouseProtocolError::InvalidCrc { expected, actual });
        }
        let mut arguments = [0u8; RAZER_MAX_ARGS];
        arguments.copy_from_slice(&slice[8..88]);
        Ok(Self {
            status: slice[0],
            transaction_id: slice[1],
            remaining_packets: u16::from_be_bytes([slice[2], slice[3]]),
            protocol_type: slice[4],
            data_size: slice[5],
            command_class: slice[6],
            command_id: slice[7],
            arguments,
        })
    }

    /// Parsed response status.
    #[must_use]
    pub const fn parsed_status(&self) -> Option<RazerReportStatus> {
        RazerReportStatus::from_byte(self.status)
    }
}

/// Compute the Razer report CRC: XOR of bytes `2..88`.
#[must_use]
pub fn compute_crc(report: &[u8]) -> u8 {
    report[2..88].iter().fold(0u8, |acc, &b| acc ^ b)
}

/// Supported mouse polling rates in Hz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PollingRate {
    Hz125 = 125,
    Hz500 = 500,
    Hz1000 = 1000,
    Hz2000 = 2000,
    Hz4000 = 4000,
    Hz8000 = 8000,
}

impl PollingRate {
    /// Convert from Hz integer.
    pub const fn from_hz(hz: u16) -> Result<Self, RazerMouseProtocolError> {
        match hz {
            125 => Ok(Self::Hz125),
            500 => Ok(Self::Hz500),
            1000 => Ok(Self::Hz1000),
            2000 => Ok(Self::Hz2000),
            4000 => Ok(Self::Hz4000),
            8000 => Ok(Self::Hz8000),
            _ => Err(RazerMouseProtocolError::UnsupportedPollingRate(hz)),
        }
    }

    /// Standard Razer polling rate wire code (`class 0x00, id 0x05`).
    #[must_use]
    pub const fn standard_code(self) -> Option<u8> {
        match self {
            Self::Hz1000 => Some(0x01),
            Self::Hz500 => Some(0x02),
            Self::Hz125 => Some(0x08),
            Self::Hz2000 | Self::Hz4000 | Self::Hz8000 => None,
        }
    }

    /// Extended HyperPolling wire code (`class 0x00, id 0x40`).
    #[must_use]
    pub const fn extended_code(self) -> u8 {
        match self {
            Self::Hz8000 => 0x01,
            Self::Hz4000 => 0x02,
            Self::Hz2000 => 0x04,
            Self::Hz1000 => 0x08,
            Self::Hz500 => 0x10,
            Self::Hz125 => 0x40,
        }
    }

    /// Decode standard polling rate wire code.
    #[must_use]
    pub const fn from_standard_code(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(Self::Hz1000),
            0x02 => Some(Self::Hz500),
            0x08 => Some(Self::Hz125),
            _ => None,
        }
    }

    /// Decode extended HyperPolling wire code.
    #[must_use]
    pub const fn from_extended_code(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(Self::Hz8000),
            0x02 => Some(Self::Hz4000),
            0x04 => Some(Self::Hz2000),
            0x08 => Some(Self::Hz1000),
            0x10 => Some(Self::Hz500),
            0x40 => Some(Self::Hz125),
            _ => None,
        }
    }
}

/// Scroll wheel mode for Razer SmartReel mice (e.g., Basilisk V3 / V3 Pro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollMode {
    Tactile = 0x00,
    FreeSpin = 0x01,
}

/// Chroma RGB lighting effect for Razer mice (`class 0x0f`, extended matrix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseLedEffect {
    Off,
    Static { r: u8, g: u8, b: u8 },
    Breathing { r: u8, g: u8, b: u8 },
    Spectrum,
    Reactive { speed: u8, r: u8, g: u8, b: u8 },
}

/// Build a "Get Firmware Version" (`0x00, 0x81`) report.
#[must_use]
pub const fn cmd_get_firmware_version(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x00, 0x81, 0x02)
}

/// Build a "Get Serial Number" (`0x00, 0x82`) report.
#[must_use]
pub const fn cmd_get_serial(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x00, 0x82, 0x16)
}

/// Build a "Get DPI XY" (`0x04, 0x85`) report.
#[must_use]
pub fn cmd_get_dpi_xy(tx: u8) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x04, 0x85, 0x07);
    r.arguments[0] = 0x01; // VARSTORE
    r
}

/// Build a "Set DPI XY" (`0x04, 0x05`) report.
pub fn cmd_set_dpi_xy(
    tx: u8,
    dpi_x: u16,
    dpi_y: u16,
) -> Result<RazerReport, RazerMouseProtocolError> {
    if !(50..=35_000).contains(&dpi_x) {
        return Err(RazerMouseProtocolError::InvalidDpi(dpi_x));
    }
    if !(50..=35_000).contains(&dpi_y) {
        return Err(RazerMouseProtocolError::InvalidDpi(dpi_y));
    }
    let x = dpi_x.to_be_bytes();
    let y = dpi_y.to_be_bytes();
    RazerReport::new(tx, 0x04, 0x05, 0x07).with_args(&[0x01, x[0], x[1], y[0], y[1], 0x00, 0x00])
}

/// Decode X/Y DPI from a `0x04, 0x85` response.
#[must_use]
pub const fn parse_dpi_xy(report: &RazerReport) -> (u16, u16) {
    let x = u16::from_be_bytes([report.arguments[1], report.arguments[2]]);
    let y = u16::from_be_bytes([report.arguments[3], report.arguments[4]]);
    (x, y)
}

/// Build a "Get Standard Polling Rate" (`0x00, 0x85`) report.
#[must_use]
pub const fn cmd_get_polling_rate(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x00, 0x85, 0x01)
}

/// Build a "Set Standard Polling Rate" (`0x00, 0x05`) report.
pub fn cmd_set_polling_rate(
    tx: u8,
    rate: PollingRate,
) -> Result<RazerReport, RazerMouseProtocolError> {
    let code = rate
        .standard_code()
        .ok_or(RazerMouseProtocolError::UnsupportedPollingRate(rate as u16))?;
    RazerReport::new(tx, 0x00, 0x05, 0x01).with_args(&[code])
}

/// Build a "Get Extended Polling Rate" (`0x00, 0xc0`) report.
#[must_use]
pub const fn cmd_get_polling_rate_ext(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x00, 0xc0, 0x02)
}

/// Build a "Set Extended Polling Rate" (`0x00, 0x40`) report.
#[must_use]
pub fn cmd_set_polling_rate_ext(tx: u8, rate: PollingRate) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x00, 0x40, 0x02);
    r.arguments[0] = 0x00;
    r.arguments[1] = rate.extended_code();
    r
}

/// Build a "Get Battery Level" (`0x07, 0x80`) report.
#[must_use]
pub const fn cmd_get_battery(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x07, 0x80, 0x02)
}

/// Convert raw battery byte (`0..=255`) to percentage (`0..=100`).
#[must_use]
pub fn parse_battery_percentage(report: &RazerReport) -> u8 {
    let raw = u16::from(report.arguments[1]);
    ((raw * 100 + 127) / 255).min(100) as u8
}

/// Build a "Get Charging Status" (`0x07, 0x84`) report.
#[must_use]
pub const fn cmd_get_charging(tx: u8) -> RazerReport {
    RazerReport::new(tx, 0x07, 0x84, 0x02)
}

/// Build a "Set Idle Time" (`0x07, 0x03`) report (seconds, clamped to `60..=900`).
#[must_use]
pub fn cmd_set_idle_time(tx: u8, seconds: u16) -> RazerReport {
    let clamped = seconds.clamp(60, 900);
    let bytes = clamped.to_be_bytes();
    let mut r = RazerReport::new(tx, 0x07, 0x03, 0x02);
    r.arguments[0] = bytes[0];
    r.arguments[1] = bytes[1];
    r
}

/// Build a "Set Low Battery Threshold" (`0x07, 0x02`) report (`percent: 5..=100`).
#[must_use]
pub fn cmd_set_low_battery_threshold(tx: u8, percent: u8) -> RazerReport {
    let pct = u16::from(percent.clamp(5, 100));
    let raw = ((pct * 255) / 100).min(255) as u8;
    let mut r = RazerReport::new(tx, 0x07, 0x02, 0x01);
    r.arguments[0] = raw;
    r
}

/// Build an extended matrix LED effect command (`0x0f, 0x02`).
///
/// `led_id = 0x00` targets all zones (`0x01` scroll wheel, `0x04` logo).
#[must_use]
pub fn cmd_set_matrix_led(tx: u8, led_id: u8, effect: MouseLedEffect) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x0f, 0x02, 0x06);
    r.arguments[0] = 0x01; // VARSTORE
    r.arguments[1] = led_id;
    match effect {
        MouseLedEffect::Off => {
            r.arguments[2] = 0x00;
        }
        MouseLedEffect::Static { r: cr, g, b } => {
            r.data_size = 0x09;
            r.arguments[2] = 0x01;
            r.arguments[5] = 0x01;
            r.arguments[6] = cr;
            r.arguments[7] = g;
            r.arguments[8] = b;
        }
        MouseLedEffect::Breathing { r: cr, g, b } => {
            r.data_size = 0x09;
            r.arguments[2] = 0x02;
            r.arguments[3] = 0x01;
            r.arguments[5] = 0x01;
            r.arguments[6] = cr;
            r.arguments[7] = g;
            r.arguments[8] = b;
        }
        MouseLedEffect::Spectrum => {
            r.arguments[2] = 0x03;
        }
        MouseLedEffect::Reactive {
            speed,
            r: cr,
            g,
            b,
        } => {
            r.data_size = 0x09;
            r.arguments[2] = 0x05;
            r.arguments[4] = speed.clamp(1, 4);
            r.arguments[5] = 0x01;
            r.arguments[6] = cr;
            r.arguments[7] = g;
            r.arguments[8] = b;
        }
    }
    r
}

/// Build an extended matrix LED brightness command (`0x0f, 0x04`).
#[must_use]
pub fn cmd_set_led_brightness(tx: u8, led_id: u8, brightness: u8) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x0f, 0x04, 0x03);
    r.arguments[0] = 0x01; // VARSTORE
    r.arguments[1] = led_id;
    r.arguments[2] = brightness;
    r
}

/// Build a "Set Scroll Mode" (`0x02, 0x14`) report (for SmartReel mice).
#[must_use]
pub fn cmd_set_scroll_mode(tx: u8, mode: ScrollMode) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x02, 0x14, 0x02);
    r.arguments[0] = 0x01; // VARSTORE
    r.arguments[1] = mode as u8;
    r
}

/// Build a "Get Scroll Mode" (`0x02, 0x94`) report.
#[must_use]
pub fn cmd_get_scroll_mode(tx: u8) -> RazerReport {
    let mut r = RazerReport::new(tx, 0x02, 0x94, 0x02);
    r.arguments[0] = 0x01;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_dpi_report_verifies_crc() {
        let req = cmd_set_dpi_xy(0x1f, 1600, 1600).expect("valid dpi");
        let encoded = req.encode();
        assert_eq!(encoded.len(), RAZER_REPORT_LEN);
        let decoded = RazerReport::decode(&encoded).expect("valid crc");
        assert_eq!(decoded.command_class, 0x04);
        assert_eq!(decoded.command_id, 0x05);
        assert_eq!(u16::from_be_bytes([decoded.arguments[1], decoded.arguments[2]]), 1600);
        assert_eq!(u16::from_be_bytes([decoded.arguments[3], decoded.arguments[4]]), 1600);
    }

    #[test]
    fn decode_rejects_bad_crc() {
        let mut encoded = cmd_get_battery(0x1f).encode();
        encoded[88] ^= 0xff;
        assert!(matches!(
            RazerReport::decode(&encoded),
            Err(RazerMouseProtocolError::InvalidCrc { .. })
        ));
    }

    #[test]
    fn battery_percentage_scales_correctly() {
        let mut r = cmd_get_battery(0x1f);
        r.arguments[1] = 255;
        assert_eq!(parse_battery_percentage(&r), 100);
        r.arguments[1] = 128;
        assert_eq!(parse_battery_percentage(&r), 50);
        r.arguments[1] = 0;
        assert_eq!(parse_battery_percentage(&r), 0);
    }
}

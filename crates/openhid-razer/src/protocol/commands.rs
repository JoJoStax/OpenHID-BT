//! Razer BLE GATT command framing and typed protocol messages.
//!
//! Commands sent to [`RAZER_BLE_COMMAND_UUID`](super::uuids::RAZER_BLE_COMMAND_UUID)
//! follow the wire layout `[op_hi, op_lo, payload_len, payload...]`.

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned when decoding a Razer BLE command packet.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RazerProtocolError {
    /// Packet has fewer than the 3-byte `[op_hi, op_lo, len]` header.
    #[error("packet too short: expected at least 3 bytes, got {0}")]
    TooShort(usize),
    /// Declared payload length does not match the remaining bytes.
    #[error("payload length mismatch: header declares {declared} bytes, got {actual}")]
    LengthMismatch { declared: usize, actual: usize },
    /// Unrecognized opcode or subcommand value.
    #[error("unsupported Razer BLE opcode {opcode:#06x} with payload {payload:?}")]
    UnsupportedCommand { opcode: u16, payload: Vec<u8> },
}

/// 24-bit RGB color for Razer Chroma BLE lighting effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RgbColor {
    /// Red channel (`0..=255`).
    pub r: u8,
    /// Green channel (`0..=255`).
    pub g: u8,
    /// Blue channel (`0..=255`).
    pub b: u8,
}

impl RgbColor {
    /// Razer signature green (`#44D62C`).
    pub const RAZER_GREEN: Self = Self {
        r: 0x44,
        g: 0xd6,
        b: 0x2c,
    };
    /// Pure red (`#FF0000`).
    pub const RED: Self = Self {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    };
    /// Pure blue (`#0000FF`).
    pub const BLUE: Self = Self {
        r: 0x00,
        g: 0x00,
        b: 0xff,
    };

    /// Construct an RGB color from channel bytes.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a 6-digit hex color string (with or without leading `#`).
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let raw = hex.trim();
        let trimmed = raw.strip_prefix('#').unwrap_or(raw);
        if trimmed.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
        let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
        let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

impl std::fmt::Display for RgbColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Supported Razer Chroma BLE lighting modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RazerLedMode {
    /// Turn earcup LEDs off (`0xc0 00 01 00`).
    Off,
    /// Turn earcup LEDs on using last active mode (`0xc0 00 01 ff`).
    On,
    /// Cycle through the rainbow spectrum (`0xc0 00 01 01`).
    Spectrum,
    /// Fade between two colors (`0xc2 00 06 R1 G1 B1 R2 G2 B2`).
    Breathing {
        /// First breathing color.
        primary: RgbColor,
        /// Second breathing color.
        secondary: RgbColor,
    },
    /// Solid single color (`0xc3 00 03 R G B`).
    Static(RgbColor),
    /// Pulse brightness with audio output amplitude (`0xc5 00 03 R G B`).
    AudioMeter(RgbColor),
}

/// Auto-standby inactivity timeout on Razer Bluetooth headsets (`0xa7 00 01 XX`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(u8)]
pub enum AutoStandby {
    /// Never power off automatically.
    Never = 0x00,
    /// Power off after 5 minutes of inactivity.
    Minutes5 = 0x01,
    /// Power off after 15 minutes of inactivity.
    Minutes15 = 0x02,
    /// Power off after 30 minutes of inactivity.
    Minutes30 = 0x03,
    /// Power off after 45 minutes of inactivity.
    Minutes45 = 0x04,
    /// Power off after 60 minutes of inactivity.
    Minutes60 = 0x05,
}

impl AutoStandby {
    /// Human-readable label for UI and CLI output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::Minutes5 => "5 min",
            Self::Minutes15 => "15 min",
            Self::Minutes30 => "30 min",
            Self::Minutes45 => "45 min",
            Self::Minutes60 => "60 min",
        }
    }
}

/// A decoded Razer BLE GATT command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RazerCommand {
    /// Chroma LED lighting command.
    Led(RazerLedMode),
    /// Auto-standby timeout command.
    AutoStandby(AutoStandby),
}

/// Build the raw ATT write payload for a [`RazerLedMode`].
#[must_use]
pub fn build_led_command(mode: RazerLedMode) -> Vec<u8> {
    match mode {
        RazerLedMode::Off => vec![0xc0, 0x00, 0x01, 0x00],
        RazerLedMode::On => vec![0xc0, 0x00, 0x01, 0xff],
        RazerLedMode::Spectrum => vec![0xc0, 0x00, 0x01, 0x01],
        RazerLedMode::Breathing { primary, secondary } => vec![
            0xc2,
            0x00,
            0x06,
            primary.r,
            primary.g,
            primary.b,
            secondary.r,
            secondary.g,
            secondary.b,
        ],
        RazerLedMode::Static(color) => vec![0xc3, 0x00, 0x03, color.r, color.g, color.b],
        RazerLedMode::AudioMeter(color) => vec![0xc5, 0x00, 0x03, color.r, color.g, color.b],
    }
}

/// Build the raw ATT write payload for an [`AutoStandby`] timeout.
#[must_use]
pub fn build_auto_standby_command(duration: AutoStandby) -> Vec<u8> {
    vec![0xa7, 0x00, 0x01, duration as u8]
}

/// Decode a raw Razer BLE GATT command packet into a typed [`RazerCommand`].
pub fn decode_command(bytes: &[u8]) -> Result<RazerCommand, RazerProtocolError> {
    if bytes.len() < 3 {
        return Err(RazerProtocolError::TooShort(bytes.len()));
    }
    let opcode = u16::from_be_bytes([bytes[0], bytes[1]]);
    let declared = usize::from(bytes[2]);
    let payload = &bytes[3..];
    if payload.len() != declared {
        return Err(RazerProtocolError::LengthMismatch {
            declared,
            actual: payload.len(),
        });
    }

    match (opcode, payload) {
        (0xc000, [0x00]) => Ok(RazerCommand::Led(RazerLedMode::Off)),
        (0xc000, [0xff]) => Ok(RazerCommand::Led(RazerLedMode::On)),
        (0xc000, [0x01]) => Ok(RazerCommand::Led(RazerLedMode::Spectrum)),
        (0xc200, [r1, g1, b1, r2, g2, b2]) => Ok(RazerCommand::Led(RazerLedMode::Breathing {
            primary: RgbColor::new(*r1, *g1, *b1),
            secondary: RgbColor::new(*r2, *g2, *b2),
        })),
        (0xc300, [r, g, b]) => Ok(RazerCommand::Led(RazerLedMode::Static(RgbColor::new(
            *r, *g, *b,
        )))),
        (0xc500, [r, g, b]) => Ok(RazerCommand::Led(RazerLedMode::AudioMeter(RgbColor::new(
            *r, *g, *b,
        )))),
        (0xa700, [raw]) => AutoStandby::try_from(*raw)
            .map(RazerCommand::AutoStandby)
            .map_err(|_| RazerProtocolError::UnsupportedCommand {
                opcode,
                payload: payload.to_vec(),
            }),
        _ => Err(RazerProtocolError::UnsupportedCommand {
            opcode,
            payload: payload.to_vec(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all_led_modes() {
        let modes = [
            RazerLedMode::Off,
            RazerLedMode::On,
            RazerLedMode::Spectrum,
            RazerLedMode::Static(RgbColor::RAZER_GREEN),
            RazerLedMode::AudioMeter(RgbColor::BLUE),
            RazerLedMode::Breathing {
                primary: RgbColor::RED,
                secondary: RgbColor::BLUE,
            },
        ];
        for mode in modes {
            let encoded = build_led_command(mode);
            let decoded = decode_command(&encoded).expect("valid LED packet");
            assert_eq!(decoded, RazerCommand::Led(mode));
        }
    }

    #[test]
    fn round_trip_auto_standby() {
        let durations = [
            AutoStandby::Never,
            AutoStandby::Minutes5,
            AutoStandby::Minutes15,
            AutoStandby::Minutes30,
            AutoStandby::Minutes45,
            AutoStandby::Minutes60,
        ];
        for duration in durations {
            let encoded = build_auto_standby_command(duration);
            let decoded = decode_command(&encoded).expect("valid standby packet");
            assert_eq!(decoded, RazerCommand::AutoStandby(duration));
        }
    }

    #[test]
    fn reject_malformed_packets() {
        assert_eq!(
            decode_command(&[0xc0, 0x00]),
            Err(RazerProtocolError::TooShort(2))
        );
        assert_eq!(
            decode_command(&[0xc3, 0x00, 0x03, 0xff]),
            Err(RazerProtocolError::LengthMismatch {
                declared: 3,
                actual: 1
            })
        );
        assert!(matches!(
            decode_command(&[0xa7, 0x00, 0x01, 0x99]),
            Err(RazerProtocolError::UnsupportedCommand { opcode: 0xa700, .. })
        ));
    }

    #[test]
    fn hex_color_parsing() {
        assert_eq!(RgbColor::from_hex("#44d62c"), Some(RgbColor::RAZER_GREEN));
        assert_eq!(RgbColor::from_hex("ff0000"), Some(RgbColor::RED));
        assert_eq!(RgbColor::from_hex("zzz"), None);
    }
}

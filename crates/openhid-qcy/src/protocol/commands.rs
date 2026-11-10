//! High-level command generators and parsers for QCY earbuds and headphones.

use serde::{Deserialize, Serialize};

use super::packet::pack_command;

/// Noise control mode (ANC, Normal, Transparency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncMode {
    Off,
    Indoor,
    Commuting,
    Noisy,
    AntiWind,
    Adaptive,
    Transparency,
}

impl AncMode {
    /// Human-readable label for UI display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Normal",
            Self::Indoor => "ANC (Indoor)",
            Self::Commuting => "ANC (Commuting)",
            Self::Noisy => "ANC (Noisy)",
            Self::AntiWind => "ANC (Anti-Wind)",
            Self::Adaptive => "ANC (Adaptive)",
            Self::Transparency => "Transparency",
        }
    }

    /// Convert ANC mode to the (mode, sub_scene, noise_value) parameter triplet for opcode `0x17`.
    #[must_use]
    pub fn to_triplet(self) -> (u8, u8, u8) {
        match self {
            Self::Adaptive => (0x01, 0x05, 0x00),
            Self::Indoor => (0x01, 0x01, 0x02),
            Self::Commuting => (0x01, 0x02, 0x02),
            Self::Noisy => (0x01, 0x03, 0x02),
            Self::AntiWind => (0x01, 0x04, 0x00),
            Self::Transparency => (0x03, 0x01, 0x04),
            Self::Off => (0x02, 0x00, 0x00),
        }
    }

    /// Parse a 3-byte acknowledgment/response into an `AncMode`.
    #[must_use]
    pub fn from_triplet(mode: u8, sub_scene: u8, _noise: u8) -> Option<Self> {
        match mode {
            0x01 => match sub_scene {
                0x01 => Some(Self::Indoor),
                0x02 => Some(Self::Commuting),
                0x03 => Some(Self::Noisy),
                0x04 => Some(Self::AntiWind),
                0x05 => Some(Self::Adaptive),
                _ => Some(Self::Adaptive),
            },
            0x02 => Some(Self::Off),
            0x03 => Some(Self::Transparency),
            _ => None,
        }
    }
}

/// Query an opcode status using the standard `0xFE` request command.
#[must_use]
pub fn build_request_command(opcode: u8) -> Vec<u8> {
    pack_command(0xFE, &[opcode])
}

/// Set active noise cancellation / transparency mode (opcode `0x17`).
#[must_use]
pub fn build_anc_command(mode: AncMode) -> Vec<u8> {
    let (m, sub, noise) = mode.to_triplet();
    pack_command(0x17, &[m, sub, noise])
}

/// Set Low Latency / Gaming Mode (opcode `0x09`).
#[must_use]
pub fn build_game_mode_command(enabled: bool) -> Vec<u8> {
    pack_command(0x09, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set Sleep Mode (opcode `0x10`).
#[must_use]
pub fn build_sleep_mode_command(enabled: bool) -> Vec<u8> {
    pack_command(0x10, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set LDAC codec toggle (opcode `0x23`).
#[must_use]
pub fn build_ldac_command(enabled: bool) -> Vec<u8> {
    pack_command(0x23, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set Dual-Device / Multipoint connection (opcode `0x24`).
#[must_use]
pub fn build_multipoint_command(enabled: bool) -> Vec<u8> {
    pack_command(0x24, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set Wind Noise Detection (opcode `0x2A`).
#[must_use]
pub fn build_wind_detection_command(enabled: bool) -> Vec<u8> {
    pack_command(0x2A, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set In-Ear Wearing Detection toggle (opcode `0x06`).
#[must_use]
pub fn build_in_ear_detection_command(enabled: bool) -> Vec<u8> {
    pack_command(0x06, &[if enabled { 0x01 } else { 0x02 }])
}

/// Set Advanced Wearing Detection (opcode `0x2C`), preserving music and anc action bytes.
#[must_use]
pub fn build_wearing_detection_command(
    enabled: bool,
    music_action: u8,
    anc_action: u8,
    tone_enabled: Option<bool>,
) -> Vec<u8> {
    let state = if enabled { 0x01 } else { 0x02 };
    match tone_enabled {
        Some(true) => pack_command(0x2C, &[state, music_action, anc_action, 0x01]),
        Some(false) => pack_command(0x2C, &[state, music_action, anc_action, 0x02]),
        None => pack_command(0x2C, &[state, music_action, anc_action]),
    }
}

/// Set Auto Power-Off timer in minutes (opcode `0x14`).
#[must_use]
pub fn build_auto_power_off_command(minutes: u16) -> Vec<u8> {
    let b0 = (minutes & 0xFF) as u8;
    let b1 = ((minutes >> 8) & 0xFF) as u8;
    pack_command(0x14, &[b0, b1, 0x00, 0x00])
}

/// Set Prompt Volume level (0..15, opcode `0x1D`).
#[must_use]
pub fn build_prompt_volume_command(volume: u8) -> Vec<u8> {
    pack_command(0x1D, &[volume.min(15)])
}

/// Select an Equalizer preset index (opcode `0x22`).
#[must_use]
pub fn build_eq_preset_command(preset_index: u8) -> Vec<u8> {
    pack_command(0x22, &[preset_index, 0x00, 0x00])
}

/// Build a 10-band parametric equalizer packet (opcode `0x22`).
/// Frequencies: 31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000 Hz.
#[must_use]
pub fn build_custom_eq_command(gains: &[f64], preset_index: u8) -> Vec<u8> {
    const FREQUENCIES: [u16; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
    let mut params = Vec::with_capacity(3 + FREQUENCIES.len() * 7);
    params.push(preset_index);
    params.push(0x00); // master gain low
    params.push(0x00); // master gain high

    for (i, &freq) in FREQUENCIES.iter().enumerate() {
        let gain_val = gains.get(i).copied().unwrap_or(0.0);
        let gain_clamped = (gain_val * 100.0).round().clamp(-800.0, 800.0) as i16;

        params.extend_from_slice(&freq.to_le_bytes());
        params.extend_from_slice(&gain_clamped.to_le_bytes());
        params.extend_from_slice(&100u16.to_le_bytes()); // Q = 1.00
        params.push(0x00); // band type
    }

    pack_command(0x22, &params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anc_mode_triplets_round_trip() {
        for mode in [
            AncMode::Adaptive,
            AncMode::Indoor,
            AncMode::Commuting,
            AncMode::Noisy,
            AncMode::AntiWind,
            AncMode::Transparency,
            AncMode::Off,
        ] {
            let (m, sub, noise) = mode.to_triplet();
            let parsed = AncMode::from_triplet(m, sub, noise);
            assert_eq!(parsed, Some(mode));
        }
    }

    #[test]
    fn builds_hardware_validated_anc_command() {
        assert_eq!(
            build_anc_command(AncMode::Adaptive),
            vec![0xFF, 0x05, 0x17, 0x03, 0x01, 0x05, 0x00]
        );
        assert_eq!(
            build_anc_command(AncMode::Off),
            vec![0xFF, 0x05, 0x17, 0x03, 0x02, 0x00, 0x00]
        );
    }

    #[test]
    fn builds_game_mode_command() {
        assert_eq!(build_game_mode_command(true), vec![0xFF, 0x03, 0x09, 0x01, 0x01]);
        assert_eq!(build_game_mode_command(false), vec![0xFF, 0x03, 0x09, 0x01, 0x02]);
    }
}

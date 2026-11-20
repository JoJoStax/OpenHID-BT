//! Sony LDAC A2DP codec (AOSP `libldac`, Vendor `0x0000012D`, Codec `0x00AA`).

use serde::{Deserialize, Serialize};

use super::{ChannelMode, SampleRate};

/// Sony Bluetooth SIG Vendor ID for LDAC.
pub const VENDOR_SONY: u32 = 0x0000_012D;
/// LDAC Vendor-Specific Codec ID.
pub const CODEC_LDAC: u16 = 0x00AA;

/// LDAC bitrate / quality preset (matching AOSP `libldac` EQMID values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LdacQuality {
    /// High Quality (`990 kbps` at 96/48 kHz, `909 kbps` at 88.2/44.1 kHz).
    #[default]
    High990,
    /// Standard Quality (`660 kbps` / `606 kbps`).
    Standard660,
    /// Mobile / Connection Priority (`330 kbps` / `303 kbps`).
    Mobile330,
    /// Adaptive Bitrate (ABR).
    Adaptive,
}

impl LdacQuality {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::High990 => "High Quality (990 kbps)",
            Self::Standard660 => "Balanced (660 kbps)",
            Self::Mobile330 => "Connection Priority (330 kbps)",
            Self::Adaptive => "Adaptive Bitrate (ABR)",
        }
    }

    /// AOSP `libldac` EQMID index (`0 = HQ`, `1 = SQ`, `2 = MQ`, `3 = ABR`).
    #[must_use]
    pub const fn eqmid(self) -> i32 {
        match self {
            Self::High990 => 0,
            Self::Standard660 => 1,
            Self::Mobile330 => 2,
            Self::Adaptive => 3,
        }
    }
}

/// Build the AVDTP Media Codec IE payload for LDAC.
#[must_use]
pub fn build_ldac_ie(sample_rate: SampleRate, channel_mode: ChannelMode) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    payload.extend_from_slice(&VENDOR_SONY.to_le_bytes());
    payload.extend_from_slice(&CODEC_LDAC.to_le_bytes());
    let sr_bit = match sample_rate {
        SampleRate::Hz96000 => 0x01,
        SampleRate::Hz88200 => 0x02,
        SampleRate::Hz48000 => 0x10,
        SampleRate::Hz44100 => 0x20,
    };
    let ch_bit = match channel_mode {
        ChannelMode::Mono => 0x04,
        ChannelMode::DualChannel => 0x02,
        ChannelMode::Stereo | ChannelMode::JointStereo => 0x01,
    };
    payload.extend_from_slice(&[sr_bit, ch_bit]);
    payload
}

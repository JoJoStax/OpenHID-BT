//! Bluetooth A2DP / AVDTP codec specifications and capability framing.
//!
//! Covers the open-source Linux/Android codec set:
//! - LDAC (AOSP `libldac`, Sony vendor `0x0000012D`, codec `0x00AA`)
//! - aptX / aptX HD / aptX LL / aptX Adaptive (`libfreeaptx` / `libopenaptx`)
//! - AAC (MPEG-2/4 AAC)
//! - SBC & SBC XQ (Dual-Channel high-bitrate baseline)

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sony Bluetooth SIG Vendor ID for LDAC.
pub const VENDOR_SONY: u32 = 0x0000_012D;
/// LDAC Vendor-Specific Codec ID.
pub const CODEC_LDAC: u16 = 0x00AA;

/// CSR / Qualcomm Bluetooth SIG Vendor ID for aptX.
pub const VENDOR_APTX: u32 = 0x0000_004F;
/// aptX Classic Codec ID.
pub const CODEC_APTX: u16 = 0x0001;

/// Qualcomm Technologies International Vendor ID for aptX HD / Adaptive.
pub const VENDOR_QUALCOMM: u32 = 0x0000_00D7;
/// aptX HD Codec ID.
pub const CODEC_APTX_HD: u16 = 0x0024;
/// aptX Adaptive Codec ID.
pub const CODEC_APTX_ADAPTIVE: u16 = 0x00AD;

/// Qualcomm / CSR Vendor ID for aptX Low Latency.
pub const VENDOR_APTX_LL: u32 = 0x0000_000A;
/// aptX Low Latency Codec ID.
pub const CODEC_APTX_LL: u16 = 0x0002;

/// Preferred Bluetooth A2DP audio codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum A2dpCodec {
    /// Automatically select the highest-quality codec supported by the sink.
    #[default]
    Auto,
    /// Sony LDAC (up to 990 kbps, 24-bit / 96 kHz).
    Ldac,
    /// Qualcomm aptX HD (576 kbps, 24-bit / 48 kHz).
    AptxHd,
    /// Qualcomm aptX Adaptive (279–420 kbps dynamic, up to 96 kHz).
    AptxAdaptive,
    /// Qualcomm aptX Low Latency (~40 ms, 352 kbps).
    AptxLl,
    /// Qualcomm aptX Classic (352 kbps, 16-bit / 48 kHz).
    Aptx,
    /// MPEG-2/4 AAC (up to 320 kbps).
    Aac,
    /// SBC XQ (Dual-Channel high-bitrate SBC, ~452–551 kbps).
    SbcXq,
    /// Standard A2DP SBC baseline (328 kbps Joint Stereo).
    Sbc,
}

impl A2dpCodec {
    /// Human-readable label for UI & CLI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto (Best Available)",
            Self::Ldac => "LDAC (990 kbps / 24-bit)",
            Self::AptxHd => "aptX HD (576 kbps / 24-bit)",
            Self::AptxAdaptive => "aptX Adaptive",
            Self::AptxLl => "aptX Low Latency",
            Self::Aptx => "aptX (352 kbps)",
            Self::Aac => "AAC (320 kbps)",
            Self::SbcXq => "SBC XQ (Dual-Channel 551 kbps)",
            Self::Sbc => "SBC (Standard)",
        }
    }

    /// Short badge name.
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Ldac => "LDAC",
            Self::AptxHd => "aptX HD",
            Self::AptxAdaptive => "aptX Adaptive",
            Self::AptxLl => "aptX LL",
            Self::Aptx => "aptX",
            Self::Aac => "AAC",
            Self::SbcXq => "SBC XQ",
            Self::Sbc => "SBC",
        }
    }

    /// Ordered preference list from highest to lowest audio fidelity.
    pub const ALL: &'static [Self] = &[
        Self::Auto,
        Self::Ldac,
        Self::AptxHd,
        Self::AptxAdaptive,
        Self::AptxLl,
        Self::Aptx,
        Self::Aac,
        Self::SbcXq,
        Self::Sbc,
    ];
}

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

/// A2DP audio sample rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SampleRate {
    Hz44100 = 44_100,
    #[default]
    Hz48000 = 48_000,
    Hz88200 = 88_200,
    Hz96000 = 96_000,
}

impl SampleRate {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hz44100 => "44.1 kHz",
            Self::Hz48000 => "48 kHz",
            Self::Hz88200 => "88.2 kHz",
            Self::Hz96000 => "96 kHz",
        }
    }

    pub const ALL: &'static [Self] = &[
        Self::Hz44100,
        Self::Hz48000,
        Self::Hz88200,
        Self::Hz96000,
    ];
}

/// A2DP channel mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChannelMode {
    Mono,
    DualChannel,
    #[default]
    Stereo,
    JointStereo,
}

/// Errors from AVDTP codec capability parsing/building.
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("AVDTP capability buffer too short ({0} bytes)")]
    BufferTooShort(usize),
    #[error("codec {0:?} has no fixed vendor-specific capability IE")]
    UnsupportedCapability(A2dpCodec),
}

/// Encoded AVDTP Media Codec Information Element (Service Category `0x07`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvdtpMediaCodecIe {
    pub codec: A2dpCodec,
    pub payload: Vec<u8>,
}

impl AvdtpMediaCodecIe {
    /// Build the AVDTP Media Codec IE bytes for configuring an A2DP sink.
    pub fn build(
        codec: A2dpCodec,
        sample_rate: SampleRate,
        channel_mode: ChannelMode,
    ) -> Result<Self, CodecError> {
        let payload = match codec {
            A2dpCodec::Ldac
            | A2dpCodec::Aptx
            | A2dpCodec::AptxHd
            | A2dpCodec::AptxLl
            | A2dpCodec::AptxAdaptive => build_vendor_ie(codec, sample_rate, channel_mode),
            A2dpCodec::Sbc | A2dpCodec::SbcXq => build_sbc_ie(codec, sample_rate),
            A2dpCodec::Aac => build_aac_ie(sample_rate),
            A2dpCodec::Auto => return Err(CodecError::UnsupportedCapability(codec)),
        };
        Ok(Self { codec, payload })
    }
}

fn build_vendor_ie(codec: A2dpCodec, sample_rate: SampleRate, channel_mode: ChannelMode) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    match codec {
        A2dpCodec::Ldac => {
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
        }
        A2dpCodec::Aptx => {
            payload.extend_from_slice(&VENDOR_APTX.to_le_bytes());
            payload.extend_from_slice(&CODEC_APTX.to_le_bytes());
            let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
            payload.push(sr_bit | 0x02);
        }
        A2dpCodec::AptxHd => {
            payload.extend_from_slice(&VENDOR_QUALCOMM.to_le_bytes());
            payload.extend_from_slice(&CODEC_APTX_HD.to_le_bytes());
            let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
            payload.extend_from_slice(&[sr_bit | 0x02, 0x00, 0x00, 0x00, 0x00]);
        }
        A2dpCodec::AptxLl => {
            payload.extend_from_slice(&VENDOR_APTX_LL.to_le_bytes());
            payload.extend_from_slice(&CODEC_APTX_LL.to_le_bytes());
            let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
            payload.extend_from_slice(&[sr_bit | 0x02, 0x00]);
        }
        A2dpCodec::AptxAdaptive => {
            payload.extend_from_slice(&VENDOR_QUALCOMM.to_le_bytes());
            payload.extend_from_slice(&CODEC_APTX_ADAPTIVE.to_le_bytes());
            let sr_bit = match sample_rate {
                SampleRate::Hz96000 => 0x08,
                SampleRate::Hz44100 => 0x20,
                _ => 0x10,
            };
            payload.extend_from_slice(&[sr_bit | 0x02, 0x00, 0x00, 0x00]);
        }
        _ => {}
    }
    payload
}

fn build_sbc_ie(codec: A2dpCodec, sample_rate: SampleRate) -> Vec<u8> {
    let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
    let ch_bit = if codec == A2dpCodec::SbcXq { 0x04 } else { 0x01 };
    let max_bitpool = if codec == A2dpCodec::SbcXq { 47 } else { 53 };
    vec![0x00, 0x00, sr_bit | ch_bit, 0x10 | 0x04 | 0x01, 2, max_bitpool]
}

fn build_aac_ie(sample_rate: SampleRate) -> Vec<u8> {
    let (sr_hi, sr_lo) = if sample_rate == SampleRate::Hz44100 {
        (0x01, 0x00)
    } else {
        (0x00, 0x80)
    };
    vec![0x00, 0x02, 0x80, sr_hi, sr_lo | 0x04, 0x80 | 0x04, 0xE2, 0x00]
}

/// Build a 12-byte RTP header for A2DP L2CAP media packets.
#[must_use]
pub fn a2dp_rtp_header(sequence: u16, timestamp: u32, payload_type: u8, ssrc: u32) -> [u8; 12] {
    let mut hdr = [0u8; 12];
    hdr[0] = 0x80; // V=2, P=0, X=0, CC=0
    hdr[1] = payload_type & 0x7F;
    hdr[2..4].copy_from_slice(&sequence.to_be_bytes());
    hdr[4..8].copy_from_slice(&timestamp.to_be_bytes());
    hdr[8..12].copy_from_slice(&ssrc.to_be_bytes());
    hdr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ldac_avdtp_capability() {
        let ie = AvdtpMediaCodecIe::build(A2dpCodec::Ldac, SampleRate::Hz96000, ChannelMode::Stereo)
            .expect("ldac ie");
        assert_eq!(ie.payload[0], 0x00);
        assert_eq!(ie.payload[1], 0xFF);
        assert_eq!(&ie.payload[2..6], &VENDOR_SONY.to_le_bytes());
        assert_eq!(&ie.payload[6..8], &CODEC_LDAC.to_le_bytes());
        assert_eq!(ie.payload[8], 0x01); // 96 kHz
        assert_eq!(ie.payload[9], 0x01); // Stereo
    }

    #[test]
    fn builds_aptx_hd_and_sbc_xq() {
        let aptx_hd =
            AvdtpMediaCodecIe::build(A2dpCodec::AptxHd, SampleRate::Hz48000, ChannelMode::Stereo)
                .expect("aptx hd ie");
        assert_eq!(&aptx_hd.payload[6..8], &CODEC_APTX_HD.to_le_bytes());

        let sbc_xq =
            AvdtpMediaCodecIe::build(A2dpCodec::SbcXq, SampleRate::Hz44100, ChannelMode::DualChannel)
                .expect("sbc xq ie");
        assert_eq!(sbc_xq.payload[2], 0x24); // 44.1kHz + DualChannel
    }
}

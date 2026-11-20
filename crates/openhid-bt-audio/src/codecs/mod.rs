//! Bluetooth A2DP / AVDTP codec specifications and capability framing.

pub mod aac;
pub mod aptx;
pub mod ldac;
pub mod sbc;

pub use ldac::LdacQuality;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
            A2dpCodec::Ldac => ldac::build_ldac_ie(sample_rate, channel_mode),
            A2dpCodec::Aptx => aptx::build_aptx_ie(sample_rate),
            A2dpCodec::AptxHd => aptx::build_aptx_hd_ie(sample_rate),
            A2dpCodec::AptxLl => aptx::build_aptx_ll_ie(sample_rate),
            A2dpCodec::AptxAdaptive => aptx::build_aptx_adaptive_ie(sample_rate),
            A2dpCodec::Sbc => sbc::build_sbc_ie(false, sample_rate),
            A2dpCodec::SbcXq => sbc::build_sbc_ie(true, sample_rate),
            A2dpCodec::Aac => aac::build_aac_ie(sample_rate),
            A2dpCodec::Auto => return Err(CodecError::UnsupportedCapability(codec)),
        };
        Ok(Self { codec, payload })
    }
}

/// Build a 12-byte RTP header for A2DP L2CAP media packets.
#[must_use]
pub fn a2dp_rtp_header(sequence: u16, timestamp: u32, payload_type: u8, ssrc: u32) -> [u8; 12] {
    let mut hdr = [0u8; 12];
    hdr[0] = 0x80;
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
        assert_eq!(&ie.payload[2..6], &ldac::VENDOR_SONY.to_le_bytes());
        assert_eq!(&ie.payload[6..8], &ldac::CODEC_LDAC.to_le_bytes());
        assert_eq!(ie.payload[8], 0x01);
        assert_eq!(ie.payload[9], 0x01);
    }

    #[test]
    fn builds_aptx_hd_and_sbc_xq() {
        let aptx_hd =
            AvdtpMediaCodecIe::build(A2dpCodec::AptxHd, SampleRate::Hz48000, ChannelMode::Stereo)
                .expect("aptx hd ie");
        assert_eq!(&aptx_hd.payload[6..8], &aptx::CODEC_APTX_HD.to_le_bytes());

        let sbc_xq =
            AvdtpMediaCodecIe::build(A2dpCodec::SbcXq, SampleRate::Hz44100, ChannelMode::DualChannel)
                .expect("sbc xq ie");
        assert_eq!(sbc_xq.payload[2], 0x24);
    }
}

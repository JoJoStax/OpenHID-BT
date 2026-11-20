//! Mandatory A2DP SBC & Dual-Channel SBC XQ codec.

use super::SampleRate;

/// Build the AVDTP Media Codec IE payload for SBC or SBC XQ.
#[must_use]
pub fn build_sbc_ie(xq: bool, sample_rate: SampleRate) -> Vec<u8> {
    let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
    let ch_bit = if xq { 0x04 } else { 0x01 };
    let max_bitpool = if xq { 47 } else { 53 };
    vec![0x00, 0x00, sr_bit | ch_bit, 0x10 | 0x04 | 0x01, 2, max_bitpool]
}

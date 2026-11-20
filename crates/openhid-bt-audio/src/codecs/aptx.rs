//! Qualcomm aptX family A2DP codecs (aptX, aptX HD, aptX LL, aptX Adaptive).

use super::SampleRate;

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

#[must_use]
pub fn build_aptx_ie(sample_rate: SampleRate) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    payload.extend_from_slice(&VENDOR_APTX.to_le_bytes());
    payload.extend_from_slice(&CODEC_APTX.to_le_bytes());
    let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
    payload.push(sr_bit | 0x02);
    payload
}

#[must_use]
pub fn build_aptx_hd_ie(sample_rate: SampleRate) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    payload.extend_from_slice(&VENDOR_QUALCOMM.to_le_bytes());
    payload.extend_from_slice(&CODEC_APTX_HD.to_le_bytes());
    let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
    payload.extend_from_slice(&[sr_bit | 0x02, 0x00, 0x00, 0x00, 0x00]);
    payload
}

#[must_use]
pub fn build_aptx_ll_ie(sample_rate: SampleRate) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    payload.extend_from_slice(&VENDOR_APTX_LL.to_le_bytes());
    payload.extend_from_slice(&CODEC_APTX_LL.to_le_bytes());
    let sr_bit = if sample_rate == SampleRate::Hz44100 { 0x20 } else { 0x10 };
    payload.extend_from_slice(&[sr_bit | 0x02, 0x00]);
    payload
}

#[must_use]
pub fn build_aptx_adaptive_ie(sample_rate: SampleRate) -> Vec<u8> {
    let mut payload = vec![0x00, 0xFF];
    payload.extend_from_slice(&VENDOR_QUALCOMM.to_le_bytes());
    payload.extend_from_slice(&CODEC_APTX_ADAPTIVE.to_le_bytes());
    let sr_bit = match sample_rate {
        SampleRate::Hz96000 => 0x08,
        SampleRate::Hz44100 => 0x20,
        _ => 0x10,
    };
    payload.extend_from_slice(&[sr_bit | 0x02, 0x00, 0x00, 0x00]);
    payload
}

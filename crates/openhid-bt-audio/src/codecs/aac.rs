//! MPEG-2/4 AAC A2DP codec (up to 320 kbps).

use super::SampleRate;

/// Build the AVDTP Media Codec IE payload for AAC.
#[must_use]
pub fn build_aac_ie(sample_rate: SampleRate) -> Vec<u8> {
    let (sr_hi, sr_lo) = if sample_rate == SampleRate::Hz44100 {
        (0x01, 0x00)
    } else {
        (0x00, 0x80)
    };
    vec![0x00, 0x02, 0x80, sr_hi, sr_lo | 0x04, 0x80 | 0x04, 0xE2, 0x00]
}

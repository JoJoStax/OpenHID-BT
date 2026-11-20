//! Declarative Bluetooth headphone & earbud capability database.
//!
//! Maps known models (QCY, Razer BT, Sony, Sennheiser, Bose, Anker Soundcore,
//! Apple) to their supported A2DP codecs, preferred codec, sample rate, and
//! on-board hardware features (ANC, Game Mode, Multipoint, On-Board EQ).

use crate::codecs::{A2dpCodec, SampleRate};

/// Capability profile for a Bluetooth headphone or earbud model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent hardware feature flags for Bluetooth audio devices"
)]
pub struct HeadphoneProfile {
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub aliases: &'static [&'static str],
    pub supported_codecs: &'static [A2dpCodec],
    pub preferred_codec: A2dpCodec,
    pub max_sample_rate: SampleRate,
    pub has_anc: bool,
    pub has_game_mode: bool,
    pub has_multipoint: bool,
    pub has_onboard_eq: bool,
}

const LDAC_SUITE: &[A2dpCodec] = &[A2dpCodec::Ldac, A2dpCodec::Aac, A2dpCodec::SbcXq, A2dpCodec::Sbc];
const APTX_HD_SUITE: &[A2dpCodec] = &[
    A2dpCodec::AptxAdaptive,
    A2dpCodec::AptxHd,
    A2dpCodec::Aptx,
    A2dpCodec::Aac,
    A2dpCodec::SbcXq,
    A2dpCodec::Sbc,
];
const AAC_SBC_SUITE: &[A2dpCodec] = &[A2dpCodec::Aac, A2dpCodec::SbcXq, A2dpCodec::Sbc];

/// Known Bluetooth headphone and earbud capabilities.
pub const HEADPHONE_DB: &[HeadphoneProfile] = &[
    // --- QCY ---
    HeadphoneProfile {
        model_id: "qcy-h3s",
        display_name: "QCY H3S",
        aliases: &["qcy h3s", "qcy-h3s", "h3s"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "qcy-h4",
        display_name: "QCY H4",
        aliases: &["qcy h4", "qcy-h4", "h4"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "qcy-h3",
        display_name: "QCY H3",
        aliases: &["qcy h3", "qcy-h3"],
        supported_codecs: AAC_SBC_SUITE,
        preferred_codec: A2dpCodec::Aac,
        max_sample_rate: SampleRate::Hz48000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "qcy-melobuds-pro",
        display_name: "QCY MeloBuds Pro",
        aliases: &["melobuds pro", "ht08"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "qcy-melobuds-n70",
        display_name: "QCY MeloBuds N70",
        aliases: &["melobuds n70", "ht18", "n70"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    // --- Razer Bluetooth ---
    HeadphoneProfile {
        model_id: "razer-opus",
        display_name: "Razer Opus",
        aliases: &["razer opus"],
        supported_codecs: &[A2dpCodec::Aptx, A2dpCodec::Aac, A2dpCodec::SbcXq, A2dpCodec::Sbc],
        preferred_codec: A2dpCodec::Aptx,
        max_sample_rate: SampleRate::Hz48000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: false,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "razer-kraken-bt",
        display_name: "Razer Kraken BT",
        aliases: &["kraken bt", "barracuda", "hammerhead", "kaira"],
        supported_codecs: AAC_SBC_SUITE,
        preferred_codec: A2dpCodec::Aac,
        max_sample_rate: SampleRate::Hz48000,
        has_anc: false,
        has_game_mode: true,
        has_multipoint: false,
        has_onboard_eq: true,
    },
    // --- Sony / Sennheiser / Bose / Soundcore / Apple ---
    HeadphoneProfile {
        model_id: "sony-wh-1000xm",
        display_name: "Sony WH/WF-1000XM",
        aliases: &["wh-1000xm", "wf-1000xm", "wh-xb", "linkbuds"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: false,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "sennheiser-momentum",
        display_name: "Sennheiser Momentum / Accentum",
        aliases: &["momentum", "accentum", "sennheiser"],
        supported_codecs: APTX_HD_SUITE,
        preferred_codec: A2dpCodec::AptxAdaptive,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "bose-qc",
        display_name: "Bose QuietComfort",
        aliases: &["bose qc", "quietcomfort", "bose nc"],
        supported_codecs: APTX_HD_SUITE,
        preferred_codec: A2dpCodec::AptxAdaptive,
        max_sample_rate: SampleRate::Hz48000,
        has_anc: true,
        has_game_mode: false,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "soundcore-ldac",
        display_name: "Anker Soundcore",
        aliases: &["soundcore", "liberty 4", "space q45", "space one"],
        supported_codecs: LDAC_SUITE,
        preferred_codec: A2dpCodec::Ldac,
        max_sample_rate: SampleRate::Hz96000,
        has_anc: true,
        has_game_mode: true,
        has_multipoint: true,
        has_onboard_eq: true,
    },
    HeadphoneProfile {
        model_id: "apple-airpods",
        display_name: "Apple AirPods",
        aliases: &["airpods"],
        supported_codecs: AAC_SBC_SUITE,
        preferred_codec: A2dpCodec::Aac,
        max_sample_rate: SampleRate::Hz48000,
        has_anc: true,
        has_game_mode: false,
        has_multipoint: false,
        has_onboard_eq: false,
    },
];

/// Generic fallback profile for unrecognized Bluetooth headphones.
pub const GENERIC_HEADPHONE_PROFILE: HeadphoneProfile = HeadphoneProfile {
    model_id: "bt-generic",
    display_name: "Bluetooth Audio Device",
    aliases: &[],
    supported_codecs: AAC_SBC_SUITE,
    preferred_codec: A2dpCodec::Aac,
    max_sample_rate: SampleRate::Hz48000,
    has_anc: false,
    has_game_mode: false,
    has_multipoint: false,
    has_onboard_eq: false,
};

/// Look up a headphone capability profile by `model_id` or advertised `name`.
#[must_use]
pub fn lookup_headphone(name: &str, model_id: Option<&str>) -> HeadphoneProfile {
    if let Some(id) = model_id
        && let Some(entry) = HEADPHONE_DB.iter().find(|e| e.model_id == id)
    {
        return entry.clone();
    }
    let lower = name.to_ascii_lowercase();
    for entry in HEADPHONE_DB {
        if entry.aliases.iter().any(|a| lower.contains(a)) {
            return entry.clone();
        }
    }
    GENERIC_HEADPHONE_PROFILE
}

/// Resolve the active codec for display (e.g. turning `Auto` into the device's
/// preferred codec from the capability DB).
#[must_use]
pub fn resolve_effective_codec(selected: A2dpCodec, profile: &HeadphoneProfile) -> A2dpCodec {
    if selected == A2dpCodec::Auto {
        profile.preferred_codec
    } else {
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_qcy_h3s_and_sony_to_ldac() {
        let h3s = lookup_headphone("QCY H3S", Some("qcy-h3s"));
        assert_eq!(h3s.preferred_codec, A2dpCodec::Ldac);
        assert_eq!(
            resolve_effective_codec(A2dpCodec::Auto, &h3s),
            A2dpCodec::Ldac
        );

        let sony = lookup_headphone("WH-1000XM5", None);
        assert_eq!(sony.preferred_codec, A2dpCodec::Ldac);
    }

    #[test]
    fn falls_back_to_aac_for_generic_headset() {
        let generic = lookup_headphone("Unknown BT Headset", None);
        assert_eq!(generic.preferred_codec, A2dpCodec::Aac);
    }
}

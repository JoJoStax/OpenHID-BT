//! Scalable product catalog and capabilities registry for QCY devices.
//!
//! Models share a single unified backend engine without per-device boilerplate.
//! Adding support for new headphones, earbuds, or open-ear devices only requires
//! registering their descriptor and capability flags in this table.

use super::commands::AncMode;

/// Form factor of a QCY audio device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QcyFormFactor {
    /// Over-ear wireless headphones (e.g. H3S, H3, H4, H2). Single battery unit.
    OverEarHeadphones,
    /// In-ear True Wireless Stereo earbuds (e.g. MeloBuds Pro, HT05, T13). Independent L/R/Case batteries.
    InEarEarbuds,
    /// Open-ear / sport earphones (e.g. Crossky GTR, Crossky Link).
    OpenEar,
}

impl std::fmt::Display for QcyFormFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverEarHeadphones => write!(f, "Over-Ear Headphones"),
            Self::InEarEarbuds => write!(f, "In-Ear Earbuds"),
            Self::OpenEar => write!(f, "Open-Ear Headphones"),
        }
    }
}

/// Standard full ANC scenes for adaptive/hybrid ANC headsets.
pub const ALL_ANC_SCENES: &[AncMode] = &[
    AncMode::Off,
    AncMode::Indoor,
    AncMode::Commuting,
    AncMode::Noisy,
    AncMode::AntiWind,
    AncMode::Adaptive,
    AncMode::Transparency,
];

/// Basic ANC scenes for budget ANC earbuds.
pub const BASIC_ANC_SCENES: &[AncMode] = &[
    AncMode::Off,
    AncMode::Indoor,
    AncMode::Transparency,
];

/// Empty ANC scenes for non-ANC headsets.
pub const NO_ANC_SCENES: &[AncMode] = &[];

/// Hardware and firmware capabilities for a QCY device model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QcyCapabilities {
    pub has_anc: bool,
    pub has_game_mode: bool,
    pub has_sleep_mode: bool,
    pub has_multipoint: bool,
    pub has_ldac: bool,
    pub has_spatial_audio: bool,
    pub has_wearing_detection: bool,
    pub has_custom_eq: bool,
    pub has_prompt_volume: bool,
    pub has_auto_power_off: bool,
    pub supported_anc_scenes: &'static [AncMode],
}

impl Default for QcyCapabilities {
    fn default() -> Self {
        Self {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        }
    }
}

/// Declarative model descriptor for a recognized QCY product family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QcyModelDescriptor {
    /// Normalized unique model identifier (e.g. "qcy-h3s", "qcy-h3", "qcy-melobuds-pro").
    pub model_id: &'static str,
    /// Canonical marketing name (e.g. "QCY H3S", "QCY H3", "QCY MeloBuds Pro").
    pub marketing_name: &'static str,
    /// Form factor (over-ear headphones vs earbuds vs open-ear).
    pub form_factor: QcyFormFactor,
    /// Hardware capabilities supported by this model.
    pub capabilities: QcyCapabilities,
    /// Bluetooth name keywords/prefixes matching this model (case-insensitive).
    pub name_aliases: &'static [&'static str],
    /// Modalias / advertised product IDs identifying this model.
    pub product_ids: &'static [u16],
}

/// Static catalog of recognized QCY headphones and earbuds.
pub const QCY_MODEL_CATALOG: &[QcyModelDescriptor] = &[
    // --- Over-Ear ANC Headphones ---
    QcyModelDescriptor {
        model_id: "qcy-h3s",
        marketing_name: "QCY H3S",
        form_factor: QcyFormFactor::OverEarHeadphones,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: true,
            has_spatial_audio: true,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        },
        name_aliases: &["QCY H3S", "QCY-H3S", "H3S"],
        product_ids: &[0x000A, 10], // modalias bluetooth:v05D6p000A
    },
    QcyModelDescriptor {
        model_id: "qcy-h3",
        marketing_name: "QCY H3",
        form_factor: QcyFormFactor::OverEarHeadphones,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: true,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        },
        name_aliases: &["QCY H3", "QCY-H3", "H3"],
        product_ids: &[0x0008, 0x0009],
    },
    QcyModelDescriptor {
        model_id: "qcy-h4",
        marketing_name: "QCY H4",
        form_factor: QcyFormFactor::OverEarHeadphones,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: true,
            has_spatial_audio: true,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        },
        name_aliases: &["QCY H4", "QCY-H4", "H4"],
        product_ids: &[0x000B],
    },
    QcyModelDescriptor {
        model_id: "qcy-h2",
        marketing_name: "QCY H2 / H2 Pro",
        form_factor: QcyFormFactor::OverEarHeadphones,
        capabilities: QcyCapabilities {
            has_anc: false,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: NO_ANC_SCENES,
        },
        name_aliases: &["QCY H2", "QCY-H2", "H2 Pro", "QCY H2 Pro"],
        product_ids: &[0x0006, 0x0007],
    },

    // --- In-Ear ANC TWS Earbuds ---
    QcyModelDescriptor {
        model_id: "qcy-melobuds-pro",
        marketing_name: "QCY MeloBuds Pro",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: true,
            has_ldac: true,
            has_spatial_audio: true,
            has_wearing_detection: true,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        },
        name_aliases: &["MeloBuds Pro", "QCY MeloBuds Pro", "HT08", "QCY HT08"],
        product_ids: &[19786],
    },
    QcyModelDescriptor {
        model_id: "qcy-melobuds-n70",
        marketing_name: "QCY MeloBuds N70",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: true,
            has_ldac: true,
            has_spatial_audio: false,
            has_wearing_detection: true,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: ALL_ANC_SCENES,
        },
        name_aliases: &["MeloBuds N70", "QCY MeloBuds N70", "HT18", "QCY HT18", "N70"],
        product_ids: &[23872, 23877],
    },
    QcyModelDescriptor {
        model_id: "qcy-melobuds-ht05",
        marketing_name: "QCY MeloBuds ANC (HT05)",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: false,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: true,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: &[
                AncMode::Off,
                AncMode::Indoor,
                AncMode::Commuting,
                AncMode::Noisy,
                AncMode::AntiWind,
                AncMode::Transparency,
            ],
        },
        name_aliases: &["MeloBuds ANC", "QCY MeloBuds ANC", "HT05", "QCY HT05", "MeloBuds"],
        product_ids: &[21020],
    },
    QcyModelDescriptor {
        model_id: "qcy-arcbuds",
        marketing_name: "QCY ArcBuds (HT07)",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: false,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: &[
                AncMode::Off,
                AncMode::Indoor,
                AncMode::Commuting,
                AncMode::Noisy,
                AncMode::AntiWind,
                AncMode::Transparency,
            ],
        },
        name_aliases: &["ArcBuds", "QCY ArcBuds", "HT07", "QCY HT07"],
        product_ids: &[21550],
    },
    QcyModelDescriptor {
        model_id: "qcy-t13-anc",
        marketing_name: "QCY T13 ANC",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: true,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: false,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: BASIC_ANC_SCENES,
        },
        name_aliases: &["T13 ANC", "QCY T13 ANC", "T13x", "QCY T13x"],
        product_ids: &[22300],
    },
    QcyModelDescriptor {
        model_id: "qcy-t13",
        marketing_name: "QCY T13",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: false,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: false,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: NO_ANC_SCENES,
        },
        name_aliases: &["QCY T13", "T13"],
        product_ids: &[20100],
    },
    QcyModelDescriptor {
        model_id: "qcy-t18",
        marketing_name: "QCY T18 / MeloBuds Neo",
        form_factor: QcyFormFactor::InEarEarbuds,
        capabilities: QcyCapabilities {
            has_anc: false,
            has_game_mode: true,
            has_sleep_mode: true,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: true,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: NO_ANC_SCENES,
        },
        name_aliases: &["QCY T18", "T18", "MeloBuds Neo", "QCY MeloBuds Neo", "T17", "QCY T17", "T20", "QCY T20"],
        product_ids: &[23155],
    },

    // --- Open-Ear Earphones ---
    QcyModelDescriptor {
        model_id: "qcy-crossky",
        marketing_name: "QCY Crossky",
        form_factor: QcyFormFactor::OpenEar,
        capabilities: QcyCapabilities {
            has_anc: false,
            has_game_mode: true,
            has_sleep_mode: false,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: true,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: NO_ANC_SCENES,
        },
        name_aliases: &["Crossky", "QCY Crossky", "Crossky GTR", "Crossky Link"],
        product_ids: &[24100],
    },
];

/// Fallback descriptor for unknown or custom-renamed QCY devices.
#[must_use]
pub fn generic_fallback_descriptor(raw_name: &str) -> QcyModelDescriptor {
    let has_anc = raw_name.to_ascii_uppercase().contains("ANC")
        || raw_name.to_ascii_uppercase().contains("PRO");

    let form_factor = if raw_name.to_ascii_uppercase().contains("HEADPHONE")
        || raw_name.starts_with("QCY H")
        || raw_name.starts_with("H")
    {
        QcyFormFactor::OverEarHeadphones
    } else {
        QcyFormFactor::InEarEarbuds
    };

    QcyModelDescriptor {
        model_id: "qcy-generic",
        marketing_name: "QCY Bluetooth Headset",
        form_factor,
        capabilities: QcyCapabilities {
            has_anc,
            has_game_mode: true,
            has_sleep_mode: form_factor == QcyFormFactor::InEarEarbuds,
            has_multipoint: true,
            has_ldac: false,
            has_spatial_audio: false,
            has_wearing_detection: false,
            has_custom_eq: true,
            has_prompt_volume: true,
            has_auto_power_off: true,
            supported_anc_scenes: if has_anc { ALL_ANC_SCENES } else { NO_ANC_SCENES },
        },
        name_aliases: &[],
        product_ids: &[],
    }
}

/// Resolve a QCY device model by matching advertised product ID, device name, and alias.
#[must_use]
pub fn resolve_model(
    raw_name: &str,
    raw_alias: &str,
    product_id: Option<u16>,
) -> QcyModelDescriptor {
    // 1. First priority: match exact hardware product ID if known
    if let Some(pid) = product_id {
        if pid != 0 {
            if let Some(desc) = QCY_MODEL_CATALOG.iter().find(|d| d.product_ids.contains(&pid)) {
                return desc.clone();
            }
        }
    }

    // 2. Second priority: match device name or alias against registered aliases
    let check_names = [raw_name.trim(), raw_alias.trim()];
    for name in check_names {
        if name.is_empty() {
            continue;
        }
        for desc in QCY_MODEL_CATALOG {
            for &alias in desc.name_aliases {
                if name.eq_ignore_ascii_case(alias)
                    || name.to_ascii_lowercase().contains(&alias.to_ascii_lowercase())
                {
                    return desc.clone();
                }
            }
        }
    }

    // 3. Fallback to generic model descriptor
    let mut generic = generic_fallback_descriptor(raw_name);
    if !raw_name.trim().is_empty() && !raw_name.trim().eq_ignore_ascii_case("QCY") {
        generic.marketing_name = "QCY Bluetooth Device";
    }
    generic
}

/// Detect product marketing name.
#[must_use]
pub fn detect_product_name(raw_name: &str, raw_alias: &str, product_id: Option<u16>) -> String {
    let model = resolve_model(raw_name, raw_alias, product_id);
    if model.model_id == "qcy-generic" {
        let trimmed = raw_name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    model.marketing_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_h3s_by_name_and_product_id() {
        let by_name = resolve_model("QCY H3S", "QCY H3S", None);
        assert_eq!(by_name.model_id, "qcy-h3s");
        assert_eq!(by_name.form_factor, QcyFormFactor::OverEarHeadphones);
        assert!(by_name.capabilities.has_anc);
        assert!(by_name.capabilities.has_game_mode);
        assert!(by_name.capabilities.has_multipoint);

        let by_pid = resolve_model("Bluetooth Headset", "Headset", Some(0x000A));
        assert_eq!(by_pid.model_id, "qcy-h3s");
    }

    #[test]
    fn resolves_h3_and_h4_correctly() {
        let h3 = resolve_model("QCY H3", "QCY H3", None);
        assert_eq!(h3.model_id, "qcy-h3");

        let h4 = resolve_model("QCY H4", "QCY H4", None);
        assert_eq!(h4.model_id, "qcy-h4");
    }

    #[test]
    fn resolves_melobuds_pro_and_n70() {
        let pro = resolve_model("QCY MeloBuds Pro", "MeloBuds Pro", None);
        assert_eq!(pro.model_id, "qcy-melobuds-pro");
        assert_eq!(pro.form_factor, QcyFormFactor::InEarEarbuds);
        assert!(pro.capabilities.has_ldac);
        assert!(pro.capabilities.has_wearing_detection);

        let n70 = resolve_model("QCY MeloBuds N70", "HT18", None);
        assert_eq!(n70.model_id, "qcy-melobuds-n70");
    }

    #[test]
    fn resolves_non_anc_t13_and_h2() {
        let t13 = resolve_model("QCY T13", "T13", None);
        assert_eq!(t13.model_id, "qcy-t13");
        assert!(!t13.capabilities.has_anc);
        assert!(t13.capabilities.has_game_mode);

        let h2 = resolve_model("QCY H2", "H2", None);
        assert_eq!(h2.model_id, "qcy-h2");
        assert!(!h2.capabilities.has_anc);
    }
}

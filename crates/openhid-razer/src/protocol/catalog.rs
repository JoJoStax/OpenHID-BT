//! Declarative product catalog and capability registry for Razer Bluetooth audio devices.

/// Form factor of a Razer Bluetooth audio device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RazerFormFactor {
    /// Over-ear wireless headphones / gaming headsets.
    OverEarHeadphones,
    /// In-ear True Wireless earbuds.
    InEarEarbuds,
}

impl std::fmt::Display for RazerFormFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverEarHeadphones => write!(f, "Over-Ear Headphones"),
            Self::InEarEarbuds => write!(f, "In-Ear Earbuds"),
        }
    }
}

/// Hardware and BLE GATT capabilities for a Razer audio model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent hardware capability flags matching Razer Audio/Chroma feature matrix"
)]
pub struct RazerCapabilities {
    /// Supports Razer Chroma BLE lighting (`416d0000-2d52-617a-6572-424c4501f40a`).
    pub has_chroma_ble: bool,
    /// Supports auto-standby inactivity timer over BLE.
    pub has_auto_standby: bool,
    /// Hardware supports Active Noise Cancellation.
    pub has_anc: bool,
    /// Hardware supports low-latency Gaming Mode.
    pub has_game_mode: bool,
    /// Hardware supports custom EQ presets.
    pub has_eq: bool,
}

impl RazerCapabilities {
    const CHROMA_HEADSET: Self = Self {
        has_chroma_ble: true,
        has_auto_standby: true,
        has_anc: false,
        has_game_mode: true,
        has_eq: true,
    };

    const ANC_HEADSET: Self = Self {
        has_chroma_ble: false,
        has_auto_standby: true,
        has_anc: true,
        has_game_mode: true,
        has_eq: true,
    };

    const CHROMA_AND_ANC: Self = Self {
        has_chroma_ble: true,
        has_auto_standby: true,
        has_anc: true,
        has_game_mode: true,
        has_eq: true,
    };

    const BASIC_HEADSET: Self = Self {
        has_chroma_ble: false,
        has_auto_standby: true,
        has_anc: false,
        has_game_mode: true,
        has_eq: true,
    };
}

/// Declarative descriptor for a recognized Razer Bluetooth audio product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RazerModelDescriptor {
    /// Normalized unique model identifier (e.g. `"razer-kraken-bt-kitty"`).
    pub model_id: &'static str,
    /// Canonical marketing name (e.g. `"Razer Kraken BT Kitty Edition"`).
    pub marketing_name: &'static str,
    /// Form factor.
    pub form_factor: RazerFormFactor,
    /// Hardware capabilities.
    pub capabilities: RazerCapabilities,
    /// Bluetooth name keywords/prefixes matching this model (case-insensitive).
    pub name_aliases: &'static [&'static str],
    /// Modalias / USB/BT product IDs identifying this model.
    pub product_ids: &'static [u16],
}

/// Known Razer Bluetooth headsets and earbuds.
pub const RAZER_MODELS: &[RazerModelDescriptor] = &[
    RazerModelDescriptor {
        model_id: "razer-kraken-bt-kitty",
        marketing_name: "Razer Kraken BT Kitty Edition",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::CHROMA_HEADSET,
        name_aliases: &["kraken bt kitty", "rz04-0352"],
        product_ids: &[0x0528, 0x0529],
    },
    RazerModelDescriptor {
        model_id: "razer-kraken-bt",
        marketing_name: "Razer Kraken BT",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::CHROMA_HEADSET,
        name_aliases: &["kraken bt"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-barracuda-v2-pro",
        marketing_name: "Razer Barracuda V2 Pro",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::ANC_HEADSET,
        name_aliases: &["barracuda v2 pro"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-barracuda-pro",
        marketing_name: "Razer Barracuda Pro",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::ANC_HEADSET,
        name_aliases: &["barracuda pro"],
        product_ids: &[0x0546],
    },
    RazerModelDescriptor {
        model_id: "razer-barracuda-x-chroma",
        marketing_name: "Razer Barracuda X Chroma",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::CHROMA_HEADSET,
        name_aliases: &["barracuda x chroma"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-barracuda-x",
        marketing_name: "Razer Barracuda X",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::BASIC_HEADSET,
        name_aliases: &["barracuda x"],
        product_ids: &[0x0544],
    },
    RazerModelDescriptor {
        model_id: "razer-barracuda",
        marketing_name: "Razer Barracuda",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::BASIC_HEADSET,
        name_aliases: &["barracuda"],
        product_ids: &[0x0545],
    },
    RazerModelDescriptor {
        model_id: "razer-opus-x",
        marketing_name: "Razer Opus X",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::ANC_HEADSET,
        name_aliases: &["opus x"],
        product_ids: &[0x053a],
    },
    RazerModelDescriptor {
        model_id: "razer-opus",
        marketing_name: "Razer Opus",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::ANC_HEADSET,
        name_aliases: &["razer opus", "opus"],
        product_ids: &[0x052d],
    },
    RazerModelDescriptor {
        model_id: "razer-hammerhead-pro-hyperspeed",
        marketing_name: "Razer Hammerhead Pro HyperSpeed",
        form_factor: RazerFormFactor::InEarEarbuds,
        capabilities: RazerCapabilities::CHROMA_AND_ANC,
        name_aliases: &["hammerhead pro hyperspeed"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-hammerhead-tws-pro",
        marketing_name: "Razer Hammerhead True Wireless Pro",
        form_factor: RazerFormFactor::InEarEarbuds,
        capabilities: RazerCapabilities::ANC_HEADSET,
        name_aliases: &["hammerhead true wireless pro", "hammerhead tws pro"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-hammerhead-tws",
        marketing_name: "Razer Hammerhead True Wireless",
        form_factor: RazerFormFactor::InEarEarbuds,
        capabilities: RazerCapabilities::CHROMA_HEADSET,
        name_aliases: &["hammerhead"],
        product_ids: &[],
    },
    RazerModelDescriptor {
        model_id: "razer-blackshark-v2-pro",
        marketing_name: "Razer BlackShark V2 Pro",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::BASIC_HEADSET,
        name_aliases: &["blackshark v2 pro", "blackshark"],
        product_ids: &[0x052c, 0x0555],
    },
    RazerModelDescriptor {
        model_id: "razer-kaira-pro",
        marketing_name: "Razer Kaira Pro",
        form_factor: RazerFormFactor::OverEarHeadphones,
        capabilities: RazerCapabilities::CHROMA_HEADSET,
        name_aliases: &["kaira pro", "kaira"],
        product_ids: &[0x0534],
    },
];

/// Fallback descriptor when a Razer Bluetooth device is recognized by vendor ID
/// or GATT service UUID without a specific model entry.
pub const GENERIC_RAZER_BT_HEADSET: RazerModelDescriptor = RazerModelDescriptor {
    model_id: "razer-bt-generic",
    marketing_name: "Razer Bluetooth Headset",
    form_factor: RazerFormFactor::OverEarHeadphones,
    capabilities: RazerCapabilities::CHROMA_HEADSET,
    name_aliases: &["razer"],
    product_ids: &[],
};

/// Resolve a Razer Bluetooth device's model descriptor from its advertised name
/// and optional modalias product ID.
#[must_use]
pub fn resolve_model(name: &str, product_id: Option<u16>) -> RazerModelDescriptor {
    if let Some(pid) = product_id
        && let Some(model) = RAZER_MODELS.iter().find(|m| m.product_ids.contains(&pid))
    {
        return model.clone();
    }

    let lower = name.to_ascii_lowercase();
    for model in RAZER_MODELS {
        if model.name_aliases.iter().any(|alias| lower.contains(alias)) {
            return model.clone();
        }
    }

    GENERIC_RAZER_BT_HEADSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_kraken_kitty_by_name() {
        let model = resolve_model("Razer Kraken BT Kitty Edition", None);
        assert_eq!(model.model_id, "razer-kraken-bt-kitty");
        assert!(model.capabilities.has_chroma_ble);
    }

    #[test]
    fn resolves_barracuda_x_by_pid() {
        let model = resolve_model("Unknown", Some(0x0544));
        assert_eq!(model.model_id, "razer-barracuda-x");
    }

    #[test]
    fn falls_back_to_generic_razer() {
        let model = resolve_model("Razer Future Audio", None);
        assert_eq!(model.model_id, "razer-bt-generic");
    }
}

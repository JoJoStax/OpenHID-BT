//! Declarative product catalog for Razer USB & 2.4GHz HyperSpeed mice.
//!
//! Product IDs and transaction IDs match OpenRazer's mouse definitions
//! (Vendor ID `0x1532`).

/// Razer USB vendor ID.
pub const RAZER_USB_VENDOR_ID: u16 = 0x1532;

/// Hardware capabilities of a Razer mouse model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent hardware capability flags for Razer mouse features"
)]
pub struct RazerMouseCapabilities {
    /// Wireless mouse or HyperSpeed dongle supporting battery/charging queries.
    pub is_wireless: bool,
    /// Supports extended HyperPolling rates (`2000`/`4000`/`8000` Hz via `0x00/0x40`).
    pub supports_hyperpolling: bool,
    /// Supports Chroma RGB matrix lighting (`0x0f/0x02`).
    pub has_chroma: bool,
    /// Supports SmartReel motorized scroll mode switching (`0x02/0x14`).
    pub has_smart_reel: bool,
    /// Maximum sensor DPI.
    pub max_dpi: u16,
    /// Default Razer report transaction ID (`0x1f` or `0x3f`).
    pub transaction_id: u8,
}

impl RazerMouseCapabilities {
    const WIRED_CHROMA: Self = Self {
        is_wireless: false,
        supports_hyperpolling: false,
        has_chroma: true,
        has_smart_reel: false,
        max_dpi: 20_000,
        transaction_id: 0x1f,
    };

    const WIRELESS_PRO: Self = Self {
        is_wireless: true,
        supports_hyperpolling: true,
        has_chroma: false,
        has_smart_reel: false,
        max_dpi: 30_000,
        transaction_id: 0x1f,
    };

    const WIRELESS_CHROMA: Self = Self {
        is_wireless: true,
        supports_hyperpolling: false,
        has_chroma: true,
        has_smart_reel: false,
        max_dpi: 20_000,
        transaction_id: 0x1f,
    };
}

/// Declarative descriptor for a Razer mouse or wireless dongle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RazerMouseDescriptor {
    pub model_id: &'static str,
    pub marketing_name: &'static str,
    pub product_id: u16,
    pub capabilities: RazerMouseCapabilities,
}

/// Known Razer mice and wireless mouse dongles (VID `0x1532`).
pub const RAZER_MICE: &[RazerMouseDescriptor] = &[
    // --- Viper Series ---
    RazerMouseDescriptor {
        model_id: "razer-viper",
        marketing_name: "Razer Viper",
        product_id: 0x0078,
        capabilities: RazerMouseCapabilities {
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-ultimate-wired",
        marketing_name: "Razer Viper Ultimate (Wired)",
        product_id: 0x007a,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-ultimate-wireless",
        marketing_name: "Razer Viper Ultimate (Wireless)",
        product_id: 0x007b,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-mini",
        marketing_name: "Razer Viper Mini",
        product_id: 0x008a,
        capabilities: RazerMouseCapabilities {
            max_dpi: 8_500,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-8khz",
        marketing_name: "Razer Viper 8KHz",
        product_id: 0x0091,
        capabilities: RazerMouseCapabilities {
            supports_hyperpolling: true,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-v2-pro-wired",
        marketing_name: "Razer Viper V2 Pro (Wired)",
        product_id: 0x00a5,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-v2-pro-wireless",
        marketing_name: "Razer Viper V2 Pro (Wireless)",
        product_id: 0x00a6,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-v3-hyperspeed",
        marketing_name: "Razer Viper V3 HyperSpeed",
        product_id: 0x00b8,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-v3-pro-wired",
        marketing_name: "Razer Viper V3 Pro (Wired)",
        product_id: 0x00c0,
        capabilities: RazerMouseCapabilities {
            max_dpi: 35_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-viper-v3-pro-wireless",
        marketing_name: "Razer Viper V3 Pro (Wireless)",
        product_id: 0x00c1,
        capabilities: RazerMouseCapabilities {
            max_dpi: 35_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    // --- DeathAdder Series ---
    RazerMouseDescriptor {
        model_id: "razer-deathadder-elite",
        marketing_name: "Razer DeathAdder Elite",
        product_id: 0x005c,
        capabilities: RazerMouseCapabilities {
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-essential",
        marketing_name: "Razer DeathAdder Essential",
        product_id: 0x006e,
        capabilities: RazerMouseCapabilities {
            max_dpi: 6_400,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-essential-2021",
        marketing_name: "Razer DeathAdder Essential (2021)",
        product_id: 0x0098,
        capabilities: RazerMouseCapabilities {
            max_dpi: 6_400,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v2",
        marketing_name: "Razer DeathAdder V2",
        product_id: 0x0084,
        capabilities: RazerMouseCapabilities::WIRED_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v2-pro-wired",
        marketing_name: "Razer DeathAdder V2 Pro (Wired)",
        product_id: 0x007c,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v2-pro-wireless",
        marketing_name: "Razer DeathAdder V2 Pro (Wireless)",
        product_id: 0x007d,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v2-mini",
        marketing_name: "Razer DeathAdder V2 Mini",
        product_id: 0x008c,
        capabilities: RazerMouseCapabilities {
            max_dpi: 8_500,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v2-x-hyperspeed",
        marketing_name: "Razer DeathAdder V2 X HyperSpeed",
        product_id: 0x009c,
        capabilities: RazerMouseCapabilities {
            has_chroma: false,
            max_dpi: 14_000,
            ..RazerMouseCapabilities::WIRELESS_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v3",
        marketing_name: "Razer DeathAdder V3",
        product_id: 0x00b2,
        capabilities: RazerMouseCapabilities {
            is_wireless: false,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v3-pro-wired",
        marketing_name: "Razer DeathAdder V3 Pro (Wired)",
        product_id: 0x00b6,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v3-pro-wireless",
        marketing_name: "Razer DeathAdder V3 Pro (Wireless)",
        product_id: 0x00b7,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v3-hyperspeed-wired",
        marketing_name: "Razer DeathAdder V3 HyperSpeed (Wired)",
        product_id: 0x00c4,
        capabilities: RazerMouseCapabilities {
            max_dpi: 26_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-deathadder-v3-hyperspeed-wireless",
        marketing_name: "Razer DeathAdder V3 HyperSpeed (Wireless)",
        product_id: 0x00c5,
        capabilities: RazerMouseCapabilities {
            max_dpi: 26_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    // --- Basilisk Series ---
    RazerMouseDescriptor {
        model_id: "razer-basilisk",
        marketing_name: "Razer Basilisk",
        product_id: 0x0064,
        capabilities: RazerMouseCapabilities {
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-essential",
        marketing_name: "Razer Basilisk Essential",
        product_id: 0x0065,
        capabilities: RazerMouseCapabilities {
            max_dpi: 6_400,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-x-hyperspeed",
        marketing_name: "Razer Basilisk X HyperSpeed",
        product_id: 0x0083,
        capabilities: RazerMouseCapabilities {
            has_chroma: false,
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRELESS_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v2",
        marketing_name: "Razer Basilisk V2",
        product_id: 0x0085,
        capabilities: RazerMouseCapabilities::WIRED_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-ultimate-wired",
        marketing_name: "Razer Basilisk Ultimate (Wired)",
        product_id: 0x0086,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-ultimate-wireless",
        marketing_name: "Razer Basilisk Ultimate (Wireless)",
        product_id: 0x0088,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3",
        marketing_name: "Razer Basilisk V3",
        product_id: 0x0099,
        capabilities: RazerMouseCapabilities {
            has_smart_reel: true,
            max_dpi: 26_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3-pro-wired",
        marketing_name: "Razer Basilisk V3 Pro (Wired)",
        product_id: 0x00aa,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3-pro-wireless",
        marketing_name: "Razer Basilisk V3 Pro (Wireless)",
        product_id: 0x00ab,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3-35k",
        marketing_name: "Razer Basilisk V3 35K",
        product_id: 0x00cb,
        capabilities: RazerMouseCapabilities {
            has_smart_reel: true,
            max_dpi: 35_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3-pro-35k-wired",
        marketing_name: "Razer Basilisk V3 Pro 35K (Wired)",
        product_id: 0x00cc,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            max_dpi: 35_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-basilisk-v3-pro-35k-wireless",
        marketing_name: "Razer Basilisk V3 Pro 35K (Wireless)",
        product_id: 0x00cd,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            max_dpi: 35_000,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    // --- Naga Series ---
    RazerMouseDescriptor {
        model_id: "razer-naga-trinity",
        marketing_name: "Razer Naga Trinity",
        product_id: 0x0067,
        capabilities: RazerMouseCapabilities {
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-pro-wired",
        marketing_name: "Razer Naga Pro (Wired)",
        product_id: 0x008f,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-pro-wireless",
        marketing_name: "Razer Naga Pro (Wireless)",
        product_id: 0x0090,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-x",
        marketing_name: "Razer Naga X",
        product_id: 0x0096,
        capabilities: RazerMouseCapabilities {
            max_dpi: 18_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-v2-pro-wired",
        marketing_name: "Razer Naga V2 Pro (Wired)",
        product_id: 0x00a7,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-v2-pro-wireless",
        marketing_name: "Razer Naga V2 Pro (Wireless)",
        product_id: 0x00a8,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            has_smart_reel: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-naga-v2-hyperspeed",
        marketing_name: "Razer Naga V2 HyperSpeed",
        product_id: 0x00b4,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
    // --- Orochi / Cobra / Atheris / Mamba / Dongle ---
    RazerMouseDescriptor {
        model_id: "razer-orochi-v2",
        marketing_name: "Razer Orochi V2",
        product_id: 0x0094,
        capabilities: RazerMouseCapabilities {
            has_chroma: false,
            max_dpi: 18_000,
            ..RazerMouseCapabilities::WIRELESS_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-cobra",
        marketing_name: "Razer Cobra",
        product_id: 0x00a3,
        capabilities: RazerMouseCapabilities {
            max_dpi: 8_500,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-cobra-pro-wired",
        marketing_name: "Razer Cobra Pro (Wired)",
        product_id: 0x00af,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-cobra-pro-wireless",
        marketing_name: "Razer Cobra Pro (Wireless)",
        product_id: 0x00b0,
        capabilities: RazerMouseCapabilities {
            has_chroma: true,
            ..RazerMouseCapabilities::WIRELESS_PRO
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-atheris",
        marketing_name: "Razer Atheris",
        product_id: 0x0062,
        capabilities: RazerMouseCapabilities {
            has_chroma: false,
            max_dpi: 7_200,
            ..RazerMouseCapabilities::WIRELESS_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-mamba-elite",
        marketing_name: "Razer Mamba Elite",
        product_id: 0x006c,
        capabilities: RazerMouseCapabilities {
            max_dpi: 16_000,
            ..RazerMouseCapabilities::WIRED_CHROMA
        },
    },
    RazerMouseDescriptor {
        model_id: "razer-mamba-wireless-wired",
        marketing_name: "Razer Mamba Wireless (Wired)",
        product_id: 0x0072,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-mamba-wireless",
        marketing_name: "Razer Mamba Wireless",
        product_id: 0x0073,
        capabilities: RazerMouseCapabilities::WIRELESS_CHROMA,
    },
    RazerMouseDescriptor {
        model_id: "razer-hyperpolling-wireless-dongle",
        marketing_name: "Razer HyperPolling Wireless Dongle",
        product_id: 0x00b3,
        capabilities: RazerMouseCapabilities::WIRELESS_PRO,
    },
];

/// Look up a known Razer mouse by USB product ID.
#[must_use]
pub fn find_razer_mouse(product_id: u16) -> Option<&'static RazerMouseDescriptor> {
    RAZER_MICE.iter().find(|m| m.product_id == product_id)
}

/// Resolve a Razer mouse descriptor by USB product ID, falling back to a generic
/// descriptor if `name` looks like a Razer mouse (or `fallback_name` is given).
#[must_use]
pub fn resolve_mouse(product_id: u16, os_name: &str) -> RazerMouseDescriptor {
    if let Some(desc) = find_razer_mouse(product_id) {
        return desc.clone();
    }
    RazerMouseDescriptor {
        model_id: "razer-mouse-generic",
        marketing_name: if os_name.trim().is_empty() {
            "Razer Mouse"
        } else {
            // Keep static fallback; caller can prefer `os_name`.
            "Razer USB Mouse"
        },
        product_id,
        capabilities: RazerMouseCapabilities {
            is_wireless: true,
            supports_hyperpolling: false,
            has_chroma: true,
            has_smart_reel: false,
            max_dpi: 30_000,
            transaction_id: 0x1f,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_mice_by_pid() {
        let viper = find_razer_mouse(0x00c1).expect("Viper V3 Pro");
        assert_eq!(viper.model_id, "razer-viper-v3-pro-wireless");
        assert!(viper.capabilities.supports_hyperpolling);
        assert_eq!(viper.capabilities.max_dpi, 35_000);

        let basilisk = find_razer_mouse(0x00ab).expect("Basilisk V3 Pro");
        assert!(basilisk.capabilities.has_smart_reel);
        assert!(basilisk.capabilities.has_chroma);
    }

    #[test]
    fn product_ids_are_unique() {
        for (i, m) in RAZER_MICE.iter().enumerate() {
            assert!(
                RAZER_MICE[..i].iter().all(|o| o.product_id != m.product_id),
                "duplicate Razer mouse PID 0x{:04x}",
                m.product_id
            );
        }
    }
}

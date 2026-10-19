//! Desktop Environment (KDE Plasma, GNOME, Hyprland, etc.) theme integration.
//!
//! Keeps OpenHID/BT's signature layout and component polish while allowing the
//! color palette, accent, surface hierarchy, and corner radii to adapt to — or
//! take inspiration from — the user's Linux desktop environment.
//!
//! Provides:
//! - Live detection of the active desktop environment (`XDG_CURRENT_DESKTOP`,
//!   `HYPRLAND_INSTANCE_SIGNATURE`, `~/.config/kdeglobals`, `~/.config/hypr/hyprland.conf`).
//! - Dynamic "Desktop Environment (Light)" and "Desktop Environment (Dark)"
//!   themes synthesized from the user's live KDE/Hyprland/GNOME color scheme.
//! - Built-in curated theme pairs for **KDE Breeze**, **GNOME Adwaita**, and
//!   **Hyprland**.

use std::fs;
use std::path::PathBuf;

/// Detected Linux desktop environment family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Kde,
    Gnome,
    Hyprland,
    Other,
}

impl DesktopEnvironment {
    /// Detect the current desktop environment from process environment variables.
    #[must_use]
    pub fn detect() -> Self {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            return Self::Hyprland;
        }
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_default()
            .to_ascii_lowercase();

        if desktop.contains("hyprland") {
            Self::Hyprland
        } else if desktop.contains("kde") || desktop.contains("plasma") {
            Self::Kde
        } else if desktop.contains("gnome")
            || desktop.contains("ubuntu")
            || desktop.contains("cosmic")
        {
            Self::Gnome
        } else {
            Self::Other
        }
    }

    /// Default Light and Dark theme names matching the detected desktop environment.
    #[must_use]
    pub const fn default_theme_pair(self) -> (&'static str, &'static str) {
        match self {
            Self::Kde => (DE_AUTO_LIGHT, DE_AUTO_DARK),
            Self::Gnome => (GNOME_ADWAITA_LIGHT, GNOME_ADWAITA_DARK),
            Self::Hyprland => (DE_AUTO_LIGHT, HYPRLAND_DARK),
            Self::Other => (super::theme::OPENLOGI_LIGHT, super::theme::OPENLOGI_DARK),
        }
    }
}

pub const DE_AUTO_LIGHT: &str = "Desktop Environment Light";
pub const DE_AUTO_DARK: &str = "Desktop Environment Dark";
pub const KDE_BREEZE_LIGHT: &str = "KDE Breeze Light";
pub const KDE_BREEZE_DARK: &str = "KDE Breeze Dark";
pub const GNOME_ADWAITA_LIGHT: &str = "GNOME Adwaita Light";
pub const GNOME_ADWAITA_DARK: &str = "GNOME Adwaita Dark";
pub const HYPRLAND_LIGHT: &str = "Hyprland Light";
pub const HYPRLAND_DARK: &str = "Hyprland Dark";

/// Extracted color palette from the user's live desktop environment config.
#[derive(Debug, Clone)]
pub struct SystemDeColors {
    pub window_bg: String,
    pub window_fg: String,
    pub view_bg: String,
    pub button_bg: String,
    pub accent: String,
    pub border: String,
    pub muted_fg: String,
    pub is_dark: bool,
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let mut p = PathBuf::from(home);
                p.push(".config");
                p
            })
        })
}

/// Parse an `"R,G,B"` string from `kdeglobals` into `(u8, u8, u8)`.
fn parse_rgb_triplet(raw: &str) -> Option<(u8, u8, u8)> {
    let mut parts = raw.trim().split(',');
    let r = parts.next()?.trim().parse::<u8>().ok()?;
    let g = parts.next()?.trim().parse::<u8>().ok()?;
    let b = parts.next()?.trim().parse::<u8>().ok()?;
    Some((r, g, b))
}

fn to_hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn blend_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let mix = |x: u8, y: u8| -> u8 {
        (f32::from(x) * (1.0 - t) + f32::from(y) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// Read live colors from `~/.config/kdeglobals` when available.
#[must_use]
pub fn read_kdeglobals_colors() -> Option<SystemDeColors> {
    let mut path = config_dir()?;
    path.push("kdeglobals");
    let content = fs::read_to_string(path).ok()?;
    parse_kdeglobals(&content)
}

#[must_use]
pub fn parse_kdeglobals(content: &str) -> Option<SystemDeColors> {
    let mut section = "";
    let mut window_bg = None;
    let mut window_fg = None;
    let mut window_inactive_fg = None;
    let mut view_bg = None;
    let mut button_bg = None;
    let mut accent = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some(sec) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = sec;
            continue;
        }
        let Some((key, val)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim();
        match (section, key) {
            ("Colors:Window", "BackgroundNormal") => window_bg = parse_rgb_triplet(val),
            ("Colors:Window", "ForegroundNormal") => window_fg = parse_rgb_triplet(val),
            ("Colors:Window", "ForegroundInactive") => window_inactive_fg = parse_rgb_triplet(val),
            ("Colors:Window", "DecorationFocus") if accent.is_none() => {
                accent = parse_rgb_triplet(val);
            }
            ("Colors:View", "BackgroundNormal") => view_bg = parse_rgb_triplet(val),
            ("Colors:Button", "BackgroundNormal") => button_bg = parse_rgb_triplet(val),
            ("Colors:Selection", "BackgroundNormal") if accent.is_none() => {
                accent = parse_rgb_triplet(val);
            }
            ("General", "AccentColor") => {
                if let Some(rgb) = parse_rgb_triplet(val) {
                    accent = Some(rgb);
                }
            }
            _ => {}
        }
    }

    let win_bg = window_bg?;
    let win_fg = window_fg.unwrap_or((236, 239, 244));
    let luma = 0.2126 * f32::from(win_bg.0)
        + 0.7152 * f32::from(win_bg.1)
        + 0.0722 * f32::from(win_bg.2);
    let is_dark = luma < 128.0;

    let v_bg = view_bg.unwrap_or_else(|| {
        if is_dark {
            blend_rgb(win_bg, (255, 255, 255), 0.05)
        } else {
            (255, 255, 255)
        }
    });
    let btn_bg = button_bg.unwrap_or_else(|| {
        if is_dark {
            blend_rgb(win_bg, (255, 255, 255), 0.10)
        } else {
            blend_rgb(win_bg, (0, 0, 0), 0.04)
        }
    });
    let acc = accent.unwrap_or((61, 174, 233)); // KDE Breeze Blue default
    let border_rgb = blend_rgb(win_bg, win_fg, 0.16);
    let muted_rgb = window_inactive_fg.unwrap_or_else(|| blend_rgb(win_bg, win_fg, 0.62));

    Some(SystemDeColors {
        window_bg: to_hex(win_bg),
        window_fg: to_hex(win_fg),
        view_bg: to_hex(v_bg),
        button_bg: to_hex(btn_bg),
        accent: to_hex(acc),
        border: to_hex(border_rgb),
        muted_fg: to_hex(muted_rgb),
        is_dark,
    })
}

/// Read live accent color from `~/.config/hypr/hyprland.conf` if present.
#[must_use]
pub fn read_hyprland_accent() -> Option<String> {
    let mut path = config_dir()?;
    path.push("hypr/hyprland.conf");
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("col.active_border")
            && let Some((_, rhs)) = trimmed.split_once('=')
        {
            if let Some(idx) = rhs.find("rgb(") {
                let rest = &rhs[idx + 4..];
                if let Some(end) = rest.find(')') {
                    let hex = rest[..end].trim().trim_start_matches("0x");
                    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Some(format!("#{hex}"));
                    }
                }
            }
            if let Some(idx) = rhs.find("rgba(") {
                let rest = &rhs[idx + 5..];
                if let Some(end) = rest.find(')') {
                    let inner = rest[..end].trim().trim_start_matches("0x");
                    if inner.len() >= 6
                        && inner[..6].chars().all(|c| c.is_ascii_hexdigit())
                    {
                        return Some(format!("#{}", &inner[..6]));
                    }
                }
            }
        }
    }
    None
}

/// Build the JSON theme bundle containing:
/// - `Desktop Environment Light` & `Desktop Environment Dark` (dynamically
///   populated from the user's live KDE `kdeglobals` / Hyprland config when
///   available, or sensible DE defaults)
/// - `KDE Breeze Light` & `KDE Breeze Dark`
/// - `GNOME Adwaita Light` & `GNOME Adwaita Dark`
/// - `Hyprland Light` & `Hyprland Dark`
#[must_use]
pub fn build_de_themes_json() -> String {
    let kde_live = read_kdeglobals_colors();
    let hypr_accent = read_hyprland_accent();

    // Determine dynamic Desktop Environment Dark colors
    let (de_dark_bg, de_dark_panel, de_dark_ctrl, de_dark_fg, de_dark_muted, de_dark_border, de_dark_accent) =
        if let Some(ref c) = kde_live {
            if c.is_dark {
                // Ensure panel is distinct from page background
                let panel = if c.view_bg == c.window_bg {
                    c.button_bg.clone()
                } else {
                    c.view_bg.clone()
                };
                (
                    c.window_bg.clone(),
                    panel,
                    c.button_bg.clone(),
                    c.window_fg.clone(),
                    c.muted_fg.clone(),
                    c.border.clone(),
                    c.accent.clone(),
                )
            } else {
                (
                    "#202326".to_string(),
                    "#292d32".to_string(),
                    "#31363b".to_string(),
                    "#eff0f1".to_string(),
                    "#939a9f".to_string(),
                    "#3b4045".to_string(),
                    c.accent.clone(),
                )
            }
        } else {
            let accent = hypr_accent
                .clone()
                .unwrap_or_else(|| "#3daee9".to_string());
            (
                "#202326".to_string(),
                "#292d32".to_string(),
                "#31363b".to_string(),
                "#eff0f1".to_string(),
                "#939a9f".to_string(),
                "#3b4045".to_string(),
                accent,
            )
        };

    // Determine dynamic Desktop Environment Light colors
    let (de_light_bg, de_light_panel, de_light_ctrl, de_light_fg, de_light_muted, de_light_border, de_light_accent) =
        if let Some(ref c) = kde_live {
            if !c.is_dark {
                (
                    c.window_bg.clone(),
                    c.view_bg.clone(),
                    c.button_bg.clone(),
                    c.window_fg.clone(),
                    c.muted_fg.clone(),
                    c.border.clone(),
                    c.accent.clone(),
                )
            } else {
                (
                    "#eff0f1".to_string(),
                    "#ffffff".to_string(),
                    "#fcfcfc".to_string(),
                    "#232629".to_string(),
                    "#5e656b".to_string(),
                    "#d5d8dc".to_string(),
                    c.accent.clone(),
                )
            }
        } else {
            let accent = hypr_accent.unwrap_or_else(|| "#3daee9".to_string());
            (
                "#eff0f1".to_string(),
                "#ffffff".to_string(),
                "#fcfcfc".to_string(),
                "#232629".to_string(),
                "#5e656b".to_string(),
                "#d5d8dc".to_string(),
                accent,
            )
        };

    let themes = serde_json::json!({
        "name": "Desktop Environments",
        "author": "OpenHID/BT",
        "themes": [
            make_theme(
                DE_AUTO_LIGHT,
                "light",
                &de_light_bg,
                &de_light_panel,
                &de_light_ctrl,
                &de_light_fg,
                &de_light_muted,
                &de_light_border,
                &de_light_accent,
            ),
            make_theme(
                DE_AUTO_DARK,
                "dark",
                &de_dark_bg,
                &de_dark_panel,
                &de_dark_ctrl,
                &de_dark_fg,
                &de_dark_muted,
                &de_dark_border,
                &de_dark_accent,
            ),
            make_theme(
                KDE_BREEZE_LIGHT,
                "light",
                "#eff0f1",
                "#ffffff",
                "#f7f8f9",
                "#232629",
                "#5e656b",
                "#d3d6da",
                "#3daee9",
            ),
            make_theme(
                KDE_BREEZE_DARK,
                "dark",
                "#202326",
                "#2a2e32",
                "#31363b",
                "#eff0f1",
                "#939a9f",
                "#3c4248",
                "#3daee9",
            ),
            make_theme(
                GNOME_ADWAITA_LIGHT,
                "light",
                "#fafafa",
                "#ffffff",
                "#f2f2f4",
                "#1e1e20",
                "#636369",
                "#dedede",
                "#3584e4",
            ),
            make_theme(
                GNOME_ADWAITA_DARK,
                "dark",
                "#1e1e1e",
                "#2a2a2a",
                "#333333",
                "#ffffff",
                "#9a9996",
                "#3d3d3d",
                "#3584e4",
            ),
            make_theme(
                HYPRLAND_LIGHT,
                "light",
                "#eff1f5",
                "#ffffff",
                "#e6e9ef",
                "#1e1e2e",
                "#5c5f77",
                "#ccd0da",
                "#00b4d8",
            ),
            make_theme(
                HYPRLAND_DARK,
                "dark",
                "#11111b",
                "#181825",
                "#1e1e2e",
                "#cdd6f4",
                "#9399b2",
                "#313244",
                "#00d2ff",
            ),
        ]
    });

    themes.to_string()
}

#[expect(
    clippy::too_many_arguments,
    reason = "theme builder maps explicit surface/text/accent roles into gpui-component JSON"
)]
fn make_theme(
    name: &str,
    mode: &str,
    bg: &str,
    panel: &str,
    control: &str,
    fg: &str,
    muted_fg: &str,
    border: &str,
    accent: &str,
) -> serde_json::Value {
    let is_dark = mode == "dark";
    let slider_thumb = if is_dark { fg } else { "#ffffff" };
    let pairs: &[(&str, &str)] = &[
        ("accent.background", control),
        ("accent.foreground", fg),
        ("accordion.background", panel),
        ("background", bg),
        ("border", border),
        ("group_box.background", panel),
        ("group_box.foreground", fg),
        ("caret", accent),
        ("danger.background", "#ef4444"),
        ("danger.foreground", "#1a1a1d"),
        ("description_list_label.background", bg),
        ("description_list_label.foreground", fg),
        ("drag_border", accent),
        ("foreground", fg),
        ("info.background", "#06b6d4"),
        ("info.foreground", "#1a1a1d"),
        ("input.border", border),
        ("link.foreground", accent),
        ("link.active.foreground", accent),
        ("link.hover.foreground", fg),
        ("list.background", panel),
        ("list.active.border", accent),
        ("muted.background", control),
        ("muted.foreground", muted_fg),
        ("popover.background", panel),
        ("popover.foreground", fg),
        ("primary.background", accent),
        ("primary.active.background", accent),
        ("primary.foreground", "#ffffff"),
        ("primary.hover.background", accent),
        ("progress_bar.background", accent),
        ("ring", accent),
        ("secondary.background", control),
        ("secondary.active.background", border),
        ("secondary.foreground", fg),
        ("secondary.hover.background", border),
        ("selection.background", accent),
        ("sidebar.background", bg),
        ("sidebar.accent.background", control),
        ("sidebar.accent.foreground", fg),
        ("sidebar.border", border),
        ("sidebar.foreground", fg),
        ("sidebar.primary.background", accent),
        ("sidebar.primary.foreground", "#ffffff"),
        ("skeleton.background", control),
        ("slider.bar.background", accent),
        ("slider.thumb.background", slider_thumb),
        ("success.background", "#22c55e"),
        ("success.foreground", "#1a1a1d"),
        ("switch.background", border),
        ("tab.background", "#00000000"),
        ("tab.active.background", panel),
        ("tab.active.foreground", fg),
        ("tab_bar.background", bg),
        ("tab_bar.segmented.background", control),
        ("tab.foreground", muted_fg),
        ("table.background", panel),
        ("table.head.foreground", muted_fg),
        ("table.row.border", border),
        ("tiles.background", bg),
        ("title_bar.background", bg),
        ("title_bar.border", border),
        ("warning.background", "#eab308"),
        ("warning.foreground", "#1a1a1d"),
        ("window.border", border),
    ];
    let colors: serde_json::Map<String, serde_json::Value> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), serde_json::Value::String((*v).to_string())))
        .collect();

    serde_json::json!({
        "is_default": false,
        "name": name,
        "mode": mode,
        "colors": colors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kdeglobals_nord_and_breeze_palettes() {
        let sample = r"
[Colors:Button]
BackgroundNormal=76,86,106

[Colors:View]
BackgroundNormal=46,52,64

[Colors:Window]
BackgroundNormal=46,52,64
ForegroundNormal=236,239,244
ForegroundInactive=216,222,233
DecorationFocus=146,110,228

[General]
AccentColor=146,110,228
";
        let parsed = parse_kdeglobals(sample).expect("should parse sample kdeglobals");
        assert!(parsed.is_dark);
        assert_eq!(parsed.window_bg, "#2e3440");
        assert_eq!(parsed.button_bg, "#4c566a");
        assert_eq!(parsed.accent, "#926ee4");
    }

    #[test]
    fn generated_de_themes_json_is_valid() {
        let json = build_de_themes_json();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("DE themes JSON must be valid");
        let themes = parsed["themes"].as_array().expect("must contain themes array");
        assert_eq!(themes.len(), 8);
    }
}

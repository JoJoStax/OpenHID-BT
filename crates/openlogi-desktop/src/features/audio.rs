//! The Audio tab of a headset: its sidetone level and Bluetooth / QCY controls.
//!
//! Sidetone plays the microphone back into the earcups, so the wearer hears
//! their own voice. The slider runs from 0 % (off) to 100 %, as HID++
//! `0x8300` documents.
//!
//! Bluetooth / QCY peripherals expose ANC modes, Game Mode (low latency),
//! LDAC codec toggles, dual-device multipoint, volume and sleep timers.

use gpui::{
    AnyElement, App, Context, IntoElement, ParentElement, Render, Styled, Subscription, Window, div, px,
};
use gpui_kit::component::{
    button::ButtonVariants as _, h_flex, slider::Slider, v_flex, Disableable as _, Icon,
    Selectable as _,
};
use openhid_qcy::protocol::commands::AncMode;
use openlogi_core::hid::SidetoneLevel;

use crate::state::{AppState, DeviceKey, SidetoneLoad, StateEvent};
use crate::ui::commit_slider::{CommitSlider, SliderRange};
use crate::ui::components::{PanelCard, Toggle, control_button, toggle_row};
use crate::ui::status::{retry_line, status_line};
use crate::ui::theme::{self, Palette, Typography as _};

/// The Audio tab's icon, from the shared action-icon set.
pub const AUDIO_ICON: &str = "action-icons/volume-2.svg";

/// The Audio tab's body, kept across frames for its slider.
pub struct AudioPanel {
    sidetone: Option<SidetoneSlider>,
    _state_obs: Subscription,
}

impl AudioPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state_obs = AppState::repaint_on(cx, |event| {
            matches!(
                event,
                StateEvent::SidetoneChanged(_) | StateEvent::AudioChanged(_)
            )
        });
        Self {
            sidetone: None,
            _state_obs: state_obs,
        }
    }
}

impl Render for AudioPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let sidetone_facts = SidetoneFacts::current(cx);
        if let Some(ref facts) = sidetone_facts {
            SidetoneSlider::sync(&mut self.sidetone, facts, window, cx);
        } else {
            self.sidetone = None;
        }

        let is_qcy_or_bt = AppState::try_read(cx)
            .and_then(AppState::current_record)
            .is_some_and(|r| {
                r.driver_id.as_deref() == Some("openhid-qcy")
                    || r.driver_id.as_deref() == Some("qcy")
                    || matches!(r.kind, openlogi_core::device::DeviceKind::Headset)
            });

        let mut panel = v_flex().w_full().gap_4();

        if let Some(facts) = sidetone_facts {
            panel = panel.child(sidetone_section(facts, self.sidetone.as_ref(), pal));
        }

        if is_qcy_or_bt {
            let audio_state = AppState::try_read(cx)
                .map(|s| s.current_audio_state())
                .unwrap_or_default();
            let online = AppState::try_read(cx)
                .and_then(AppState::current_record)
                .is_some_and(|r| r.is_online());

            panel = panel
                .child(render_anc_card(audio_state.anc_mode, online, pal))
                .child(render_performance_card(
                    audio_state.game_mode,
                    audio_state.multipoint,
                    audio_state.ldac,
                    online,
                    pal,
                ))
                .child(render_hardware_card(
                    audio_state.prompt_volume,
                    audio_state.auto_power_off_minutes,
                    online,
                    pal,
                ));
        }

        panel
    }
}

/// What the sidetone row needs about the selected headset.
struct SidetoneFacts {
    key: DeviceKey,
    load: SidetoneLoad,
    reachable: bool,
}

impl SidetoneFacts {
    /// The selected headset's facts, or `None` when nothing is selected or
    /// the device has no `0x8300`.
    fn current(cx: &App) -> Option<Self> {
        let state = AppState::try_read(cx)?;
        if !state.current_sidetone_supported() {
            return None;
        }
        let record = state.current_record()?;
        Some(Self {
            key: record.device_key(),
            load: state.current_sidetone_load(),
            reachable: state.current_sidetone_route().is_some(),
        })
    }

    /// The level to put the slider on, or `None` while the slider must not
    /// show: the headset is offline, or its reading has not landed.
    fn slider_level(&self) -> Option<SidetoneLevel> {
        match &self.load {
            SidetoneLoad::Ready(level) if self.reachable => Some(**level),
            _ => None,
        }
    }
}

/// The slider, kept across frames, with the headset it was built for.
struct SidetoneSlider {
    key: DeviceKey,
    slider: CommitSlider<SidetoneLevel>,
}

impl SidetoneSlider {
    /// Keep `current` in step with `facts`: drop the slider when it must not
    /// show, build one for a newly selected headset, or move the thumb when
    /// the level changed underneath it.
    fn sync<V: 'static>(
        current: &mut Option<Self>,
        facts: &SidetoneFacts,
        window: &mut Window,
        cx: &mut Context<V>,
    ) {
        let Some(level) = facts.slider_level() else {
            *current = None;
            return;
        };
        if let Some(existing) = current
            .as_ref()
            .filter(|existing| existing.key == facts.key)
        {
            existing.slider.sync(level, window, cx);
            return;
        }
        let slider = CommitSlider::new(
            SliderRange::new(SidetoneLevel::OFF, SidetoneLevel::MAX),
            level,
            cx,
            // `commit_sidetone` resolves the headset when the thumb is
            // released, so a device switch mid-drag writes the headset now
            // on screen.
            |_, level, cx| AppState::apply(cx, |state| state.commit_sidetone(level)),
        );
        *current = Some(Self {
            key: facts.key.clone(),
            slider,
        });
    }
}

/// The row: a header with the level as a percent, the slider or a status
/// line, and what the setting does.
fn sidetone_section(
    facts: SidetoneFacts,
    slider: Option<&SidetoneSlider>,
    pal: Palette,
) -> AnyElement {
    let shown = facts
        .slider_level()
        .map(|level| slider.map_or(level, |slider| slider.slider.shown(level)));
    let SidetoneFacts {
        key,
        load,
        reachable,
    } = facts;
    let body: AnyElement = match (load, slider) {
        _ if !reachable => {
            status_line(tr!("audio.device_offline_sidetone_is_unavailable"), pal).into_any_element()
        }
        (SidetoneLoad::Ready(_), Some(slider)) => Slider::new(slider.slider.slider())
            .horizontal()
            .into_any_element(),
        (SidetoneLoad::Ready(_), None) | (SidetoneLoad::Unknown | SidetoneLoad::Loading, _) => {
            status_line(tr!("audio.reading_sidetone"), pal).into_any_element()
        }
        (SidetoneLoad::Failed(_), _) => retry_line(
            "sidetone-retry",
            tr!("audio.couldnt_read_sidetone_click_to_retry"),
            move |cx| AppState::apply(cx, |state| state.retry_sidetone_read(&key)),
        )
        .into_any_element(),
        (SidetoneLoad::Unsupported(_), _) => {
            status_line(tr!("audio.sidetone_unsupported"), pal).into_any_element()
        }
    };
    v_flex()
        .gap_3()
        .w_full()
        .child(
            h_flex()
                .justify_between()
                .items_baseline()
                .child(
                    div()
                        .text_body()
                        .text_color(pal.text_muted)
                        .child(tr!("audio.sidetone")),
                )
                .children(shown.map(|level| {
                    div()
                        .text_body()
                        .text_color(pal.text_primary)
                        .child(tr!("audio.sidetone_percent", percent => level.percent()))
                })),
        )
        .child(body)
        .child(
            div()
                .text_caption()
                .text_color(pal.text_muted)
                .child(tr!("audio.sidetone_description")),
        )
        .into_any_element()
}

fn render_anc_card(current_mode: AncMode, online: bool, pal: Palette) -> impl IntoElement {
    let is_anc = matches!(
        current_mode,
        AncMode::Adaptive
            | AncMode::Indoor
            | AncMode::Commuting
            | AncMode::Noisy
            | AncMode::AntiWind
    );
    let is_transparency = matches!(current_mode, AncMode::Transparency);
    let is_off = matches!(current_mode, AncMode::Off);

    let mode_buttons = h_flex()
        .w_full()
        .gap_2()
        .child({
            let mut btn = control_button("anc-off")
                .flex_1()
                .label("Normal (Off)")
                .disabled(!online)
                .on_click(|_, _, cx| {
                    AppState::apply(cx, |s| s.commit_audio_anc(AncMode::Off));
                });
            if is_off {
                btn = btn.primary();
            }
            btn
        })
        .child({
            let mut btn = control_button("anc-active")
                .flex_1()
                .label("Noise Cancelling")
                .disabled(!online)
                .on_click(|_, _, cx| {
                    AppState::apply(cx, |s| s.commit_audio_anc(AncMode::Adaptive));
                });
            if is_anc {
                btn = btn.primary();
            }
            btn
        })
        .child({
            let mut btn = control_button("anc-transparency")
                .flex_1()
                .label("Transparency")
                .disabled(!online)
                .on_click(|_, _, cx| {
                    AppState::apply(cx, |s| s.commit_audio_anc(AncMode::Transparency));
                });
            if is_transparency {
                btn = btn.primary();
            }
            btn
        });

    let scenes_row = is_anc.then(|| {
        v_flex()
            .gap_2()
            .pt_2()
            .border_t_1()
            .border_color(pal.border)
            .child(
                div()
                    .text_caption()
                    .text_color(pal.text_muted)
                    .child("ANC Tuning Scene"),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_1p5()
                    .flex_wrap()
                    .child(scene_button(
                        "scene-adaptive",
                        "Adaptive",
                        matches!(current_mode, AncMode::Adaptive),
                        AncMode::Adaptive,
                        online,
                    ))
                    .child(scene_button(
                        "scene-indoor",
                        "Indoor",
                        matches!(current_mode, AncMode::Indoor),
                        AncMode::Indoor,
                        online,
                    ))
                    .child(scene_button(
                        "scene-commuting",
                        "Commuting",
                        matches!(current_mode, AncMode::Commuting),
                        AncMode::Commuting,
                        online,
                    ))
                    .child(scene_button(
                        "scene-noisy",
                        "Noisy / Crowd",
                        matches!(current_mode, AncMode::Noisy),
                        AncMode::Noisy,
                        online,
                    ))
                    .child(scene_button(
                        "scene-antiwind",
                        "Anti-Wind",
                        matches!(current_mode, AncMode::AntiWind),
                        AncMode::AntiWind,
                        online,
                    )),
            )
    });

    PanelCard::new(
        "Active Noise Cancellation",
        Icon::empty().path("action-icons/volume-2.svg"),
        v_flex()
            .gap_3()
            .child(
                div()
                    .text_caption()
                    .text_color(pal.text_muted)
                    .child("Toggle ambient sound suppression or transparency passthrough."),
            )
            .child(mode_buttons)
            .children(scenes_row),
    )
}

fn scene_button(
    id: &'static str,
    label: &'static str,
    selected: bool,
    mode: AncMode,
    online: bool,
) -> impl IntoElement {
    let mut btn = control_button(id)
        .label(label)
        .disabled(!online)
        .on_click(move |_, _, cx| {
            AppState::apply(cx, |s| s.commit_audio_anc(mode));
        });
    if selected {
        btn = btn.primary();
    }
    btn
}

fn render_performance_card(
    game_mode: bool,
    multipoint: bool,
    ldac: bool,
    online: bool,
    pal: Palette,
) -> impl IntoElement {
    let game_mode_row = toggle_row(
        "game-mode-row",
        pal,
        "Low-Latency Game Mode".into(),
        "Reduces Bluetooth audio latency down to ~68ms for synchronized gaming and video.".into(),
        Toggle::new("game-mode-toggle")
            .selected(game_mode)
            .disabled(!online)
            .label((!online).then(|| tr!("common.unavailable")))
            .on_change(|enabled, _window, cx| {
                AppState::apply(cx, |s| s.commit_audio_game_mode(*enabled));
            }),
    );

    let ldac_row = toggle_row(
        "ldac-row",
        pal,
        "LDAC High-Resolution Codec".into(),
        "Enables 990 kbps high-bitrate streaming for lossless audio fidelity.".into(),
        Toggle::new("ldac-toggle")
            .selected(ldac)
            .disabled(!online)
            .label((!online).then(|| tr!("common.unavailable")))
            .on_change(|enabled, _window, cx| {
                AppState::apply(cx, |s| s.commit_audio_ldac(*enabled));
            }),
    );

    let multipoint_row = toggle_row(
        "multipoint-row",
        pal,
        "Dual-Device Multipoint".into(),
        "Maintains concurrent connections with two host devices simultaneously.".into(),
        Toggle::new("multipoint-toggle")
            .selected(multipoint)
            .disabled(!online)
            .label((!online).then(|| tr!("common.unavailable")))
            .on_change(|enabled, _window, cx| {
                AppState::apply(cx, |s| s.commit_audio_multipoint(*enabled));
            }),
    );

    PanelCard::new(
        "Audio & Performance",
        Icon::empty().path("action-icons/gauge.svg"),
        v_flex()
            .gap_4()
            .child(game_mode_row)
            .child(ldac_row)
            .child(multipoint_row),
    )
}

fn render_hardware_card(
    prompt_volume: u8,
    auto_off_minutes: u16,
    online: bool,
    pal: Palette,
) -> impl IntoElement {
    let volume_row = h_flex()
        .justify_between()
        .items_center()
        .gap_4()
        .child(
            v_flex()
                .flex_1()
                .child(div().text_body().child("Voice Prompt Volume"))
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child("Adjust volume level of built-in spoken notification prompts (1 - 15)."),
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    control_button("vol-down")
                        .label("-")
                        .disabled(!online || prompt_volume <= 1)
                        .on_click(move |_, _, cx| {
                            let next = prompt_volume.saturating_sub(1).max(1);
                            AppState::apply(cx, |s| s.commit_audio_prompt_volume(next));
                        }),
                )
                .child(
                    div()
                        .min_w(px(28.))
                        .text_center()
                        .text_subheading()
                        .child(prompt_volume.to_string()),
                )
                .child(
                    control_button("vol-up")
                        .label("+")
                        .disabled(!online || prompt_volume >= 15)
                        .on_click(move |_, _, cx| {
                            let next = prompt_volume.saturating_add(1).min(15);
                            AppState::apply(cx, |s| s.commit_audio_prompt_volume(next));
                        }),
                ),
        );

    let timeouts = [(0, "Disabled"), (10, "10 min"), (20, "20 min"), (30, "30 min"), (60, "60 min")];
    let auto_off_row = v_flex()
        .gap_2()
        .pt_2()
        .border_t_1()
        .border_color(pal.border)
        .child(
            v_flex()
                .child(div().text_body().child("Auto Power-Off (Sleep Timer)"))
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child("Automatically power down the headset when disconnected to preserve battery."),
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .children(timeouts.into_iter().map(|(mins, label)| {
                    let mut btn = control_button(format!("auto-off-{mins}"))
                        .label(label)
                        .disabled(!online)
                        .on_click(move |_, _, cx| {
                            AppState::apply(cx, |s| s.commit_audio_power_off(mins));
                        });
                    if auto_off_minutes == mins {
                        btn = btn.primary();
                    }
                    btn
                })),
        );

    PanelCard::new(
        "Device Configuration & Power",
        Icon::empty().path("action-icons/settings.svg"),
        v_flex().gap_4().child(volume_row).child(auto_off_row),
    )
}

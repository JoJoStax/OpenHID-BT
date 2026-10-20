//! Audio & ANC controls for Bluetooth / QCY headsets and earbuds.

use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Subscription, Window, div, px,
};
use gpui_component::{
    Disableable as _, Icon, Selectable as _, button::ButtonVariants as _, h_flex, v_flex,
};
use openhid_qcy::protocol::commands::AncMode;

use crate::state::{AppState, StateEvent};
use crate::ui::components::{PanelCard, Toggle, control_button, toggle_row};
use crate::ui::theme::{self, Palette, Typography as _};

pub struct AudioPanel {
    _state_obs: Subscription,
}

impl AudioPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state_obs =
            AppState::repaint_on(cx, |event| matches!(event, StateEvent::AudioChanged(_)));
        Self {
            _state_obs: state_obs,
        }
    }
}

impl Render for AudioPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let audio_state = AppState::try_read(cx)
            .map(|s| s.current_audio_state())
            .unwrap_or_default();

        let online = AppState::try_read(cx)
            .and_then(AppState::current_record)
            .is_some_and(|r| r.online);

        v_flex()
            .w_full()
            .gap_4()
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
            ))
    }
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
                        "Crowded / Noisy",
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
        "Noise Control (ANC)",
        Icon::empty().path("action-icons/volume-2.svg"),
        v_flex()
            .gap_3()
            .child(
                div()
                    .text_body()
                    .text_color(pal.text_muted)
                    .child("Switch active acoustic isolation modes between Hybrid ANC, environmental Transparency, and Standard."),
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
        "Low Latency Game Mode".into(),
        "Minimizes Bluetooth audio delay down to ~68ms for gaming and real-time video sync.".into(),
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

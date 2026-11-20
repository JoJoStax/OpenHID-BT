//! Bluetooth Audio & Windows A2DP HD Codec Driver settings page.

use openhid_bt_audio::{
    WindowsDriverStatus, check_driver_status, install_or_update_driver,
    is_driver_setup_applicable, probe_codec_libraries, uninstall_driver,
};

use super::{
    Button, ButtonVariants, Entity, IconName, ParentElement, SettingField,
    SettingGroup, SettingItem, SettingPage, SettingsView, Sizable, Styled, Tag, h_flex,
    v_flex,
};

pub(super) fn bluetooth_page(view: Entity<SettingsView>) -> SettingPage {
    let driver = check_driver_status();
    let codecs = probe_codec_libraries();

    let mut page = SettingPage::new("Bluetooth Audio")
        .icon(IconName::Settings2)
        .default_open(true);

    if is_driver_setup_applicable() {
        let (status_label, primary_btn_label) = match driver.status {
            WindowsDriverStatus::NotInstalled => ("Not Installed", "Install Driver"),
            WindowsDriverStatus::Installed => ("Installed", "Reinstall / Repair"),
            WindowsDriverStatus::UpdateAvailable => ("Update Available", "Update Driver"),
            WindowsDriverStatus::NotApplicable => ("Native OS Stack", "Install"),
        };

        let installed_desc = match driver.installed_version {
            Some(ref v) => format!(
                "OpenHID/BT Windows A2DP HD Audio Driver v{v} (bundled v{}). Enables LDAC, aptX HD, aptX Adaptive, aptX LL, and SBC XQ on Windows.",
                driver.bundled_version
            ),
            None => format!(
                "Stock Windows BthA2dp.sys is active (no LDAC / aptX HD). Install OpenHID/BT A2DP Driver v{} for Linux/Android-grade codecs.",
                driver.bundled_version
            ),
        };

        let view_install = view.clone();
        let view_uninstall = view.clone();
        let can_uninstall = matches!(
            driver.status,
            WindowsDriverStatus::Installed | WindowsDriverStatus::UpdateAvailable
        );

        page = page.group(
            SettingGroup::new()
                .title("Windows Bluetooth A2DP Driver")
                .description("Replaces stock BthA2dp.sys with open-source LDAC and aptX codec encoders.")
                .item(
                    SettingItem::new(
                        "Driver Status",
                        SettingField::render(move |_options, _window, _cx| {
                            let tag = match driver.status {
                                WindowsDriverStatus::Installed => Tag::success().child(status_label),
                                WindowsDriverStatus::UpdateAvailable => Tag::warning().child(status_label),
                                _ => Tag::secondary().child(status_label),
                            };
                            let mut actions = h_flex().gap_2().items_center().child(tag).child(
                                Button::new("bt-driver-install")
                                    .small()
                                    .primary()
                                    .label(primary_btn_label)
                                    .on_click({
                                        let view = view_install.clone();
                                        move |_, _, cx| {
                                            let _ = install_or_update_driver();
                                            view.update(cx, |_, cx| cx.notify());
                                        }
                                    }),
                            );
                            if can_uninstall {
                                actions = actions.child(
                                    Button::new("bt-driver-uninstall")
                                        .small()
                                        .label("Restore Windows Default")
                                        .on_click({
                                            let view = view_uninstall.clone();
                                            move |_, _, cx| {
                                                let _ = uninstall_driver();
                                                view.update(cx, |_, cx| cx.notify());
                                            }
                                        }),
                                );
                            }
                            actions
                        }),
                    )
                    .description(installed_desc),
                ),
        );
    } else {
        page = page.group(
            SettingGroup::new()
                .title("Bluetooth Audio Stack")
                .item(
                    SettingItem::new(
                        "Native OS Driver Active",
                        SettingField::render(|_options, _window, _cx| {
                            Tag::success().child("Native BlueZ / PipeWire")
                        }),
                    )
                    .description(
                        "Linux and macOS handle A2DP codecs natively — no custom Windows A2DP driver setup is needed.",
                    ),
                ),
        );
    }

    page.group(
        SettingGroup::new()
            .title("Open-Source Codec Encoders")
            .description("Availability of AOSP libldac, libfreeaptx, AAC, and SBC encoders on this system.")
            .item(
                SettingItem::new(
                    "LDAC (AOSP libldac)",
                    SettingField::render(move |_, _, _| {
                        if codecs.ldac_available {
                            Tag::success().child("Available (990 / 660 / 330 kbps)")
                        } else {
                            Tag::secondary().child("Not Detected")
                        }
                    }),
                )
                .description("24-bit / 96 kHz high-resolution Bluetooth audio encoding."),
            )
            .item(
                SettingItem::new(
                    "aptX / aptX HD / aptX LL (libfreeaptx)",
                    SettingField::render(move |_, _, _| {
                        if codecs.aptx_available {
                            Tag::success().child("Available")
                        } else {
                            Tag::secondary().child("Not Detected")
                        }
                    }),
                )
                .description("Open-source Qualcomm aptX family encoder library."),
            )
            .item(
                SettingItem::new(
                    "AAC & SBC XQ",
                    SettingField::render(move |_, _, _| {
                        v_flex().child(if codecs.aac_available && codecs.sbc_available {
                            Tag::success().child("Available")
                        } else {
                            Tag::secondary().child("SBC Baseline")
                        })
                    }),
                )
                .description("320 kbps AAC and Dual-Channel SBC XQ (551 kbps)."),
            ),
    )
}

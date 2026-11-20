//! Optimistic audio settings and BlueZ GATT dispatch for Bluetooth/QCY peripherals.

use openhid_qcy::protocol::commands::AncMode;

use super::device_key::DeviceKey;
use super::device_session::AudioDeviceState;
use super::events::{StateEvent, StateEvents};
use super::AppState;

impl AppState {
    /// Return the audio state for a specific device key.
    pub fn audio_state_for(&self, key: &DeviceKey) -> AudioDeviceState {
        self.devices
            .sessions
            .get(key)
            .map(|s| s.audio.clone())
            .unwrap_or_default()
    }

    /// Return the active device's audio state.
    pub fn current_audio_state(&self) -> AudioDeviceState {
        let Some(key) = self.current_record().map(|r| r.device_key()) else {
            return AudioDeviceState::default();
        };
        self.audio_state_for(&key)
    }

    /// Set ANC mode and dispatch over BlueZ.
    pub fn commit_audio_anc(&mut self, mode: AncMode) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .anc_mode = mode;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_anc(mode).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set Game Mode (low latency) and dispatch over BlueZ.
    pub fn commit_audio_game_mode(&mut self, enabled: bool) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .game_mode = enabled;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_game_mode(enabled).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set Dual-Device Multipoint and dispatch over BlueZ.
    pub fn commit_audio_multipoint(&mut self, enabled: bool) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .multipoint = enabled;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_multipoint(enabled).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set LDAC high-res codec and dispatch over BlueZ.
    pub fn commit_audio_ldac(&mut self, enabled: bool) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .ldac = enabled;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_ldac(enabled).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set prompt volume level (1..=15).
    pub fn commit_audio_prompt_volume(&mut self, volume: u8) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .prompt_volume = volume;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_prompt_volume(volume).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set auto-power-off timer in minutes.
    pub fn commit_audio_power_off(&mut self, minutes: u16) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key.clone())
            .or_default()
            .audio
            .auto_power_off_minutes = minutes;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_auto_power_off(minutes).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set Razer Chroma BLE LED mode and dispatch over BlueZ.
    pub fn commit_razer_led(
        &mut self,
        mode: openhid_razer::protocol::commands::RazerLedMode,
    ) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key)
            .or_default()
            .audio
            .razer_led_mode = mode;

        dispatch_razer_action(address, move |client| async move {
            let _ = client.set_led_mode(mode).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set Razer Bluetooth headset auto-standby timeout and dispatch over BlueZ.
    pub fn commit_razer_standby(
        &mut self,
        duration: openhid_razer::protocol::commands::AutoStandby,
    ) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key)
            .or_default()
            .audio
            .razer_standby = duration;

        dispatch_razer_action(address, move |client| async move {
            let _ = client.set_auto_standby(duration).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set preferred Bluetooth A2DP codec (LDAC, aptX HD, aptX Adaptive, AAC, SBC XQ, etc.).
    pub fn commit_audio_codec(&mut self, codec: openhid_bt_audio::A2dpCodec) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        let audio = &mut self.devices.sessions.entry(key).or_default().audio;
        audio.a2dp_codec = codec;
        let ldac_enabled = matches!(codec, openhid_bt_audio::A2dpCodec::Ldac);
        audio.ldac = ldac_enabled;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_ldac(ldac_enabled).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set LDAC bitrate / quality preset (990 / 660 / 330 kbps / ABR).
    pub fn commit_ldac_quality(&mut self, quality: openhid_bt_audio::LdacQuality) -> StateEvents {
        let Some((key, _)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key)
            .or_default()
            .audio
            .ldac_quality = quality;

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set preferred A2DP sample rate (44.1 / 48 / 88.2 / 96 kHz).
    pub fn commit_sample_rate(&mut self, rate: openhid_bt_audio::SampleRate) -> StateEvents {
        let Some((key, _)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key)
            .or_default()
            .audio
            .sample_rate = rate;

        self.for_current_device(StateEvent::AudioChanged)
    }

    /// Set on-board equalizer preset index and dispatch to the headset.
    pub fn commit_eq_preset(&mut self, preset: u8) -> StateEvents {
        let Some((key, address)) = self.current_audio_target() else {
            return StateEvents::none();
        };
        self.devices
            .sessions
            .entry(key)
            .or_default()
            .audio
            .eq_preset = preset;

        dispatch_qcy_action(address, move |client| async move {
            let _ = client.set_eq_preset(preset).await;
        });

        self.for_current_device(StateEvent::AudioChanged)
    }

    fn current_audio_target(&self) -> Option<(DeviceKey, Option<String>)> {
        let record = self.current_record()?;
        let key = record.device_key();
        let address = record.serial_number.clone();
        Some((key, address))
    }
}

fn dispatch_qcy_action<F, Fut>(address: Option<String>, f: F)
where
    F: FnOnce(openhid_qcy::transport::BluezQcyClient) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async move {
                if let Ok(devs) = openhid_qcy::transport::enumerate_bluez_qcy_devices().await {
                    let target = if let Some(ref addr) = address {
                        devs.into_iter().find(|d| d.address.eq_ignore_ascii_case(addr))
                    } else {
                        devs.into_iter().find(|d| d.connected)
                    };
                    if let Some(dev) = target {
                        let client = openhid_qcy::transport::BluezQcyClient::new(dev);
                        f(client).await;
                    }
                }
            });
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = address;
        let _ = f;
    }
}

fn dispatch_razer_action<F, Fut>(address: Option<String>, f: F)
where
    F: FnOnce(openhid_razer::transport::BluezRazerClient) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async move {
                if let Ok(devs) = openhid_razer::transport::enumerate_bluez_razer_devices().await {
                    let target = if let Some(ref addr) = address {
                        devs.into_iter().find(|d| d.address.eq_ignore_ascii_case(addr))
                    } else {
                        devs.into_iter().find(|d| d.connected)
                    };
                    if let Some(dev) = target {
                        let client = openhid_razer::transport::BluezRazerClient::new(dev);
                        f(client).await;
                    }
                }
            });
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = address;
        let _ = f;
    }
}

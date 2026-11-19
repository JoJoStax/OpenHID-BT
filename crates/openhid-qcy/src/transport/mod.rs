//! Bluetooth transports for QCY devices.

#[cfg(target_os = "linux")]
pub mod bluez;

#[cfg(target_os = "linux")]
pub use bluez::{BluezQcyClient, DiscoveredQcyDevice, enumerate_bluez_qcy_devices, send_qcy_packet};

#[cfg(not(target_os = "linux"))]
pub mod dummy {
    use crate::protocol::catalog::{QcyModelDescriptor, generic_fallback_descriptor};
    use crate::protocol::commands::AncMode;

    #[derive(Debug, Clone)]
    pub struct DiscoveredQcyDevice {
        pub address: String,
        pub name: String,
        pub alias: String,
        pub connected: bool,
        pub paired: bool,
        pub battery_percentage: Option<u8>,
        pub model: QcyModelDescriptor,
    }

    impl Default for DiscoveredQcyDevice {
        fn default() -> Self {
            Self {
                address: String::new(),
                name: String::new(),
                alias: String::new(),
                connected: false,
                paired: false,
                battery_percentage: None,
                model: generic_fallback_descriptor("QCY"),
            }
        }
    }

    pub async fn enumerate_bluez_qcy_devices() -> Result<Vec<DiscoveredQcyDevice>, String> {
        Ok(Vec::new())
    }

    #[derive(Clone)]
    pub struct BluezQcyClient {
        pub device: DiscoveredQcyDevice,
    }

    impl BluezQcyClient {
        #[must_use]
        pub const fn new(device: DiscoveredQcyDevice) -> Self {
            Self { device }
        }

        pub async fn set_anc(&self, _mode: AncMode) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_game_mode(&self, _enabled: bool) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_multipoint(&self, _enabled: bool) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_ldac(&self, _enabled: bool) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_prompt_volume(&self, _volume: u8) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_auto_power_off(&self, _minutes: u16) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_eq_preset(&self, _preset_index: u8) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_custom_eq(&self, _gains: &[f64], _preset_index: u8) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use dummy::*;

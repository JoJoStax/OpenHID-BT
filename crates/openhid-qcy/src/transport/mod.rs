//! Bluetooth transports for QCY devices.

#[cfg(target_os = "linux")]
pub mod bluez;

#[cfg(target_os = "linux")]
pub use bluez::{BluezQcyClient, DiscoveredQcyDevice, enumerate_bluez_qcy_devices, send_qcy_packet};

#[cfg(not(target_os = "linux"))]
pub mod dummy {
    use crate::protocol::commands::AncMode;

    #[derive(Debug, Clone)]
    pub struct DiscoveredQcyDevice {
        pub address: String,
        pub name: String,
        pub alias: String,
        pub connected: bool,
        pub paired: bool,
        pub battery_percentage: Option<u8>,
    }

    pub async fn enumerate_bluez_qcy_devices() -> Result<Vec<DiscoveredQcyDevice>, String> {
        Ok(Vec::new())
    }

    #[derive(Clone)]
    pub struct BluezQcyClient {
        pub device: DiscoveredQcyDevice,
    }

    impl BluezQcyClient {
        pub async fn set_anc(&self, _mode: AncMode) -> Result<(), String> {
            Ok(())
        }
        pub async fn set_game_mode(&self, _enabled: bool) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use dummy::*;

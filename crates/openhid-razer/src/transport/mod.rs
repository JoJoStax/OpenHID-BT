//! Bluetooth transports for Razer devices.

#[cfg(target_os = "linux")]
pub mod bluez;

#[cfg(target_os = "linux")]
pub use bluez::{
    BluezRazerClient, DiscoveredRazerDevice, enumerate_bluez_razer_devices, send_razer_packet,
};

#[cfg(not(target_os = "linux"))]
pub mod dummy {
    use crate::protocol::catalog::{GENERIC_RAZER_BT_HEADSET, RazerModelDescriptor};
    use crate::protocol::commands::{AutoStandby, RazerLedMode};

    /// Stub discovered Razer device on non-Linux hosts.
    #[derive(Debug, Clone)]
    pub struct DiscoveredRazerDevice {
        /// Bluetooth MAC address.
        pub address: String,
        /// Advertised device name.
        pub name: String,
        /// BlueZ alias.
        pub alias: String,
        /// Whether the device is connected.
        pub connected: bool,
        /// Whether the device is paired.
        pub paired: bool,
        /// Battery percentage.
        pub battery_percentage: Option<u8>,
        /// Model descriptor.
        pub model: RazerModelDescriptor,
    }

    impl Default for DiscoveredRazerDevice {
        fn default() -> Self {
            Self {
                address: String::new(),
                name: String::new(),
                alias: String::new(),
                connected: false,
                paired: false,
                battery_percentage: None,
                model: GENERIC_RAZER_BT_HEADSET,
            }
        }
    }

    /// Enumerate Razer Bluetooth devices (no-op off Linux).
    pub async fn enumerate_bluez_razer_devices() -> Result<Vec<DiscoveredRazerDevice>, String> {
        Ok(Vec::new())
    }

    /// Stub BlueZ client on non-Linux hosts.
    #[derive(Clone)]
    pub struct BluezRazerClient {
        /// Discovered device metadata.
        pub device: DiscoveredRazerDevice,
    }

    impl BluezRazerClient {
        /// Wrap a discovered device.
        #[must_use]
        pub const fn new(device: DiscoveredRazerDevice) -> Self {
            Self { device }
        }

        /// Set Chroma BLE LED mode (no-op off Linux).
        pub async fn set_led_mode(&self, _mode: RazerLedMode) -> Result<(), String> {
            Ok(())
        }

        /// Set auto-standby timeout (no-op off Linux).
        pub async fn set_auto_standby(&self, _duration: AutoStandby) -> Result<(), String> {
            Ok(())
        }

        /// Write raw payload (no-op off Linux).
        pub async fn write_raw(&self, _payload: &[u8]) -> Result<(), String> {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use dummy::*;

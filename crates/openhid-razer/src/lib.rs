//! OpenHID/BT Razer Bluetooth headset protocol and backend crate.
//!
//! Provides BLE GATT framing, Chroma lighting and standby commands, product
//! catalog resolution, and Linux BlueZ discovery for Razer Bluetooth headsets
//! (Kraken BT, Barracuda, Opus, Hammerhead, BlackShark, Kaira).

pub mod device;
pub mod mouse;
pub mod protocol;
pub mod transport;

pub use device::{RAZER_DRIVER_ID, RazerDeviceState, standalone_device_from_razer};
pub use protocol::commands::{
    AutoStandby, RazerCommand, RazerLedMode, RazerProtocolError, RgbColor,
    build_auto_standby_command, build_led_command, decode_command,
};
pub use transport::{BluezRazerClient, DiscoveredRazerDevice, enumerate_bluez_razer_devices};

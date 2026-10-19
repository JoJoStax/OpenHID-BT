//! OpenHID/BT QCY Bluetooth backend and protocol crate.
//!
//! Provides protocol framing, commands, and Bluetooth (BlueZ) discovery for
//! QCY wireless earbuds and headphones (MeloBuds Pro, N70, H3, H3S, etc.).

pub mod device;
pub mod protocol;
pub mod transport;

pub use device::{QCY_DRIVER_ID, QcyDeviceState, standalone_device_from_qcy};
pub use protocol::commands::{AncMode, build_anc_command, build_game_mode_command};
pub use transport::{BluezQcyClient, DiscoveredQcyDevice, enumerate_bluez_qcy_devices};

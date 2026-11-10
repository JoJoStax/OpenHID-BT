//! What this GUI session knows about one device and never persists.

use openlogi_core::device::ModelTransport;
use openlogi_core::hid::ReceiverProtocol;

use super::disable_keys::DisableKeysDeviceState;
use super::light::LightDeviceState;
use super::smartshift::SmartShiftDeviceState;

/// An observed connection mode, without a hardware address or write authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservedConnection {
    /// A directly attached USB or Bluetooth HID++ device.
    Direct(ModelTransport),
    /// A paired device reached through this receiver protocol.
    Receiver(ReceiverProtocol),
}

/// Everything `AppState` tracks per device outside persisted configuration and
/// the swr-backed DPI/SmartShift reads: the per-device counterpart of
/// [`AgentSession`](super::agent::AgentSession), gone with the process.
///
/// Replaces parallel `BTreeMap<String, _>` fields that all shared the same
/// device-key domain — manual camera-light override, volatile light
/// settings, an in-flight light command, a SmartShift write lifecycle, and
/// its confirmation status — with one row per device. The row also keeps
/// light-command status scoped to the device that produced it. A device
/// absent from the owning map is equivalent to every field here at its
/// default. How many snapshots in a row omitted a device is not here: it is
/// the record's own [`Presence`](super::devices::Presence).
#[derive(Debug, Default)]
pub(super) struct DeviceSession {
    /// Last observed connection mode, for offline presentation only.
    pub(super) last_connection: Option<ObservedConnection>,
    pub(super) smartshift: SmartShiftDeviceState,
    pub(super) light: LightDeviceState,
    pub(super) disable_keys: DisableKeysDeviceState,
    pub(super) audio: AudioDeviceState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AudioDeviceState {
    pub(crate) anc_mode: openhid_qcy::protocol::commands::AncMode,
    pub(crate) game_mode: bool,
    pub(crate) ldac: bool,
    pub(crate) multipoint: bool,
    pub(crate) prompt_volume: u8,
    pub(crate) auto_power_off_minutes: u16,
}

impl Default for AudioDeviceState {
    fn default() -> Self {
        Self {
            anc_mode: openhid_qcy::protocol::commands::AncMode::Off,
            game_mode: false,
            ldac: false,
            multipoint: false,
            prompt_volume: 8,
            auto_power_off_minutes: 0,
        }
    }
}

//! Cross-platform Bluetooth A2DP codec definitions, AVDTP capability
//! framing, open-source encoder probing, and Windows A2DP driver management.

pub mod codecs;
pub mod encoder;
pub mod headphone_db;
pub mod windows_driver;

pub use codecs::{
    A2dpCodec, AvdtpMediaCodecIe, ChannelMode, LdacQuality, SampleRate, a2dp_rtp_header,
};
pub use encoder::{CodecLibraryStatus, probe_codec_libraries};
pub use headphone_db::{
    GENERIC_HEADPHONE_PROFILE, HEADPHONE_DB, HeadphoneProfile, lookup_headphone,
    resolve_effective_codec,
};
pub use windows_driver::{
    BUNDLED_DRIVER_VERSION, WindowsDriverInfo, WindowsDriverStatus, check_driver_status,
    install_or_update_driver, is_driver_setup_applicable, uninstall_driver,
};

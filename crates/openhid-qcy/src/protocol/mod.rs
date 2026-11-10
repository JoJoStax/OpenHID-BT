//! Protocol definitions and framing for QCY Bluetooth devices.

pub mod advertisement;
pub mod catalog;
pub mod commands;
pub mod packet;
pub mod uuids;

pub use catalog::{
    QCY_MODEL_CATALOG, QcyCapabilities, QcyFormFactor, QcyModelDescriptor, detect_product_name,
    resolve_model,
};
pub use commands::{AncMode, build_anc_command, build_game_mode_command, build_request_command};
pub use packet::{QcyCommand, pack_command, parse_packet};
pub use uuids::{
    HELE_QCY_VENDOR_ID, QCY_BATTERY_UUID, QCY_COMMAND_UUID, QCY_COMPANY_ID, QCY_NOTIFICATION_UUID,
    QCY_SERVICE_UUID, is_qcy_name, is_qcy_vendor,
};

//! Razer USB & 2.4GHz HyperSpeed mouse protocol, catalog, and HID transport.

pub mod catalog;
pub mod device;
pub mod report;
pub mod transport;

pub use catalog::{
    RAZER_MICE, RAZER_USB_VENDOR_ID, RazerMouseCapabilities, RazerMouseDescriptor, find_razer_mouse,
    resolve_mouse,
};
pub use device::{RAZER_MOUSE_DRIVER_ID, standalone_device_from_razer_mouse};
pub use report::{
    MouseLedEffect, PollingRate, RAZER_REPORT_LEN, RazerMouseProtocolError, RazerReport,
    RazerReportStatus, ScrollMode,
};
pub use transport::{DiscoveredRazerMouse, RazerMouseClient, RazerMouseError, enumerate_razer_mice};

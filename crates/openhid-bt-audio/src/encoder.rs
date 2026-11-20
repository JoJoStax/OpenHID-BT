//! Open-source Bluetooth codec library probing (`libldac`, `libfreeaptx` /
//! `libopenaptx`, AAC, SBC).
//!
//! On Linux, PipeWire / BlueZ loads `libldacBT_enc.so` and `libfreeaptx.so`.
//! On Windows, the OpenHID/BT A2DP driver package ships `ldacBT_enc.dll` and
//! `freeaptx.dll` (built from AOSP `platform/external/libldac` and `libfreeaptx`)
//! alongside the driver directory.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Availability status of the open-source A2DP codec encoders.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent open-source encoder library availability flags"
)]
pub struct CodecLibraryStatus {
    pub ldac_available: bool,
    pub aptx_available: bool,
    pub aac_available: bool,
    pub sbc_available: bool,
}

/// Probe whether the open-source codec encoders are present on this host.
#[must_use]
pub fn probe_codec_libraries() -> CodecLibraryStatus {
    #[cfg(target_os = "linux")]
    {
        let has_lib = |names: &[&str]| -> bool {
            let dirs = [
                "/usr/lib",
                "/usr/lib64",
                "/usr/lib/x86_64-linux-gnu",
                "/usr/lib/aarch64-linux-gnu",
                "/usr/local/lib",
                "/run/current-system/sw/lib",
            ];
            dirs.iter().any(|dir| {
                names
                    .iter()
                    .any(|n| PathBuf::from(dir).join(n).exists())
            })
        };
        CodecLibraryStatus {
            ldac_available: has_lib(&["libldacBT_enc.so", "libldacBT_enc.so.2"]),
            aptx_available: has_lib(&["libfreeaptx.so", "libfreeaptx.so.0", "libopenaptx.so"]),
            aac_available: has_lib(&["libfdk-aac.so", "libfdk-aac.so.2"]),
            sbc_available: true,
        }
    }

    #[cfg(target_os = "windows")]
    {
        let driver_dir = crate::windows_driver::driver_install_dir();
        let has_dll = |names: &[&str]| -> bool {
            names.iter().any(|n| {
                driver_dir.as_ref().is_some_and(|d| d.join(n).exists())
                    || PathBuf::from(r"C:\Windows\System32").join(n).exists()
            })
        };
        CodecLibraryStatus {
            ldac_available: has_dll(&["ldacBT_enc.dll", "libldacBT_enc.dll"]),
            aptx_available: has_dll(&["freeaptx.dll", "libfreeaptx.dll", "openaptx.dll"]),
            aac_available: true, // Windows Media Foundation AAC encoder
            sbc_available: true,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = PathBuf::new();
        CodecLibraryStatus {
            ldac_available: false,
            aptx_available: false,
            aac_available: true,
            sbc_available: true,
        }
    }
}

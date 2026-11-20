//! Windows Alternative A2DP-style Bluetooth Audio Driver manager.
//!
//! On Linux (BlueZ + PipeWire) and macOS (CoreAudio), custom A2DP driver
//! installation is redundant. On Windows, stock `BthA2dp.sys` lacks LDAC and
//! aptX HD/Adaptive, so OpenHID/BT provides an in-app setup/update/uninstall
//! flow for its Windows Bluetooth A2DP Codec Driver (`OpenHidBthA2dp`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version of the bundled Windows A2DP Codec Driver package.
pub const BUNDLED_DRIVER_VERSION: &str = "0.9.0";

/// Installation state of the Windows Bluetooth A2DP Codec Driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WindowsDriverStatus {
    /// Platform does not need a custom A2DP driver (Linux / macOS).
    #[default]
    NotApplicable,
    /// Windows is running with stock `BthA2dp.sys`; custom driver not installed.
    NotInstalled,
    /// Custom OpenHID/BT A2DP driver is installed and up to date.
    Installed,
    /// An older version of the custom A2DP driver is installed; update available.
    UpdateAvailable,
}

/// Detailed status of the Windows A2DP driver and open-source codec DLLs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WindowsDriverInfo {
    pub status: WindowsDriverStatus,
    pub installed_version: Option<String>,
    pub bundled_version: String,
    pub driver_dir: Option<PathBuf>,
}

/// Errors from Windows driver inspection or installation.
#[derive(Debug, Error)]
pub enum WindowsDriverError {
    #[error("Windows Bluetooth A2DP driver setup is only applicable on Windows")]
    UnsupportedPlatform,
    #[error("driver package files not found at {0}")]
    PackageNotFound(String),
    #[error("failed to launch elevated driver installer: {0}")]
    LaunchFailed(String),
}

/// Returns `true` only on Windows (on Linux/macOS it is redundant).
#[must_use]
pub const fn is_driver_setup_applicable() -> bool {
    cfg!(target_os = "windows")
}

/// Directory where the Windows A2DP driver & codec DLLs are staged.
#[must_use]
pub fn driver_install_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join("OpenHID-BT").join("drivers").join("bth-a2dp"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Locate the bundled `packaging/windows/bth-a2dp-driver` directory next to the
/// executable or in the workspace tree.
#[must_use]
pub fn locate_driver_package_dir() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let sibling = dir.join("drivers").join("bth-a2dp");
        if sibling.join("OpenHidBthA2dp.inf").exists() {
            return Some(sibling);
        }
    }
    let dev = PathBuf::from("packaging/windows/bth-a2dp-driver");
    if dev.join("OpenHidBthA2dp.inf").exists() {
        return Some(dev);
    }
    None
}

/// Inspect whether the OpenHID/BT Windows A2DP driver is installed.
#[must_use]
pub fn check_driver_status() -> WindowsDriverInfo {
    if !is_driver_setup_applicable() {
        return WindowsDriverInfo {
            status: WindowsDriverStatus::NotApplicable,
            installed_version: None,
            bundled_version: BUNDLED_DRIVER_VERSION.to_string(),
            driver_dir: None,
        };
    }

    let dir = driver_install_dir();
    let version_marker = dir.as_ref().map(|d| d.join("VERSION"));
    let installed_version = version_marker
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let status = match installed_version.as_deref() {
        Some(v) if v == BUNDLED_DRIVER_VERSION => WindowsDriverStatus::Installed,
        Some(_) => WindowsDriverStatus::UpdateAvailable,
        None => WindowsDriverStatus::NotInstalled,
    };

    WindowsDriverInfo {
        status,
        installed_version,
        bundled_version: BUNDLED_DRIVER_VERSION.to_string(),
        driver_dir: dir,
    }
}

/// Install or update the Windows A2DP Codec Driver (elevated via UAC).
pub fn install_or_update_driver() -> Result<(), WindowsDriverError> {
    #[cfg(not(target_os = "windows"))]
    {
        Err(WindowsDriverError::UnsupportedPlatform)
    }

    #[cfg(target_os = "windows")]
    {
        let pkg = locate_driver_package_dir().ok_or_else(|| {
            WindowsDriverError::PackageNotFound("drivers/bth-a2dp/OpenHidBthA2dp.inf".to_string())
        })?;
        let script = pkg.join("install-driver.ps1");
        if !script.exists() {
            return Err(WindowsDriverError::PackageNotFound(
                script.display().to_string(),
            ));
        }

        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\" -Version \"{}\"'",
                    script.display(),
                    BUNDLED_DRIVER_VERSION
                ),
            ])
            .status()
            .map_err(|e| WindowsDriverError::LaunchFailed(e.to_string()))?;

        if !status.success() {
            return Err(WindowsDriverError::LaunchFailed(format!(
                "installer exited with {status}"
            )));
        }
        Ok(())
    }
}

/// Uninstall the OpenHID/BT A2DP driver and restore Windows' default `BthA2dp.sys`.
pub fn uninstall_driver() -> Result<(), WindowsDriverError> {
    #[cfg(not(target_os = "windows"))]
    {
        Err(WindowsDriverError::UnsupportedPlatform)
    }

    #[cfg(target_os = "windows")]
    {
        let pkg = locate_driver_package_dir().ok_or_else(|| {
            WindowsDriverError::PackageNotFound("drivers/bth-a2dp/uninstall-driver.ps1".to_string())
        })?;
        let script = pkg.join("uninstall-driver.ps1");
        if !script.exists() {
            return Err(WindowsDriverError::PackageNotFound(
                script.display().to_string(),
            ));
        }

        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"'",
                    script.display()
                ),
            ])
            .status()
            .map_err(|e| WindowsDriverError::LaunchFailed(e.to_string()))?;

        if !status.success() {
            return Err(WindowsDriverError::LaunchFailed(format!(
                "uninstaller exited with {status}"
            )));
        }
        Ok(())
    }
}

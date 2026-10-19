use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub enum QcyCmd {
    /// List discovered QCY Bluetooth devices with detected model & capabilities.
    List,
    /// Inspect details and full hardware capabilities of a QCY device.
    Info(DeviceFilterArgs),
    /// Set Active Noise Cancellation / Transparency mode.
    Anc(AncArgs),
    /// Turn Low Latency (Game Mode) on or off.
    GameMode(GameModeArgs),
    /// Turn Multipoint (Dual-Device connection) on or off.
    Multipoint(ToggleArgs),
    /// Turn LDAC high-resolution audio codec on or off.
    Ldac(ToggleArgs),
    /// Set Prompt Volume level (0..15).
    PromptVolume(PromptVolumeArgs),
    /// Set Auto Power-Off timer in minutes.
    AutoOff(AutoOffArgs),
}

#[derive(Debug, Args)]
pub struct DeviceFilterArgs {
    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliAncMode {
    Off,
    Indoor,
    Commuting,
    Noisy,
    #[value(name = "anti-wind", alias = "antiwind")]
    AntiWind,
    Adaptive,
    Transparency,
}

#[derive(Debug, Args)]
pub struct AncArgs {
    /// ANC mode to set.
    #[arg(value_enum)]
    pub mode: CliAncMode,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct GameModeArgs {
    /// Enable or disable Game Mode (low latency).
    #[arg(action = clap::ArgAction::Set)]
    pub enabled: bool,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct ToggleArgs {
    /// Enable or disable this feature.
    #[arg(action = clap::ArgAction::Set)]
    pub enabled: bool,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct PromptVolumeArgs {
    /// Volume level from 0 to 15.
    #[arg(value_parser = clap::value_parser!(u8))]
    pub volume: u8,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct AutoOffArgs {
    /// Minutes before auto-power off.
    #[arg(value_parser = clap::value_parser!(u16))]
    pub minutes: u16,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

impl QcyCmd {
    pub async fn run(self) -> Result<ExitCode> {
        #[cfg(not(target_os = "linux"))]
        {
            bail!("QCY Bluetooth support is currently only available on Linux via BlueZ.");
        }

        #[cfg(target_os = "linux")]
        {
            match self {
                Self::List => run_list().await,
                Self::Info(args) => run_info(args).await,
                Self::Anc(args) => run_anc(args).await,
                Self::GameMode(args) => run_game_mode(args).await,
                Self::Multipoint(args) => run_multipoint(args).await,
                Self::Ldac(args) => run_ldac(args).await,
                Self::PromptVolume(args) => run_prompt_volume(args).await,
                Self::AutoOff(args) => run_auto_off(args).await,
            }
        }
    }
}

#[cfg(target_os = "linux")]
async fn run_list() -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to query BlueZ for QCY devices")?;

    if devices.is_empty() {
        println!("No QCY Bluetooth devices found in BlueZ.");
        println!("Ensure your QCY device is paired via bluetoothctl or your desktop settings.");
        return Ok(ExitCode::SUCCESS);
    }

    println!("Found {} QCY Bluetooth device(s):", devices.len());
    for dev in &devices {
        let conn_status = if dev.connected { "connected" } else { "disconnected" };
        let paired_status = if dev.paired { "paired" } else { "not paired" };
        let battery = dev
            .battery_percentage
            .map_or_else(|| "unknown".to_string(), |b| format!("{b}%"));
        let has_cmd = if dev.command_char_path.is_some() { "yes" } else { "no" };

        let mut caps = Vec::new();
        if dev.model.capabilities.has_anc {
            caps.push("ANC");
        }
        if dev.model.capabilities.has_game_mode {
            caps.push("Game Mode");
        }
        if dev.model.capabilities.has_multipoint {
            caps.push("Multipoint");
        }
        if dev.model.capabilities.has_ldac {
            caps.push("LDAC");
        }
        if dev.model.capabilities.has_custom_eq {
            caps.push("10-Band EQ");
        }

        println!();
        println!("{} [{}]", dev.model.marketing_name, dev.address);
        println!("  Form factor: {}", dev.model.form_factor);
        println!("  Alias: {}", dev.alias);
        println!("  State: {conn_status}, {paired_status}");
        println!("  Battery: {battery}");
        println!("  Capabilities: {}", caps.join(", "));
        println!("  GATT command ready: {has_cmd}");
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_info(args: DeviceFilterArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to query BlueZ for QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    println!("Model: {}", target.model.marketing_name);
    println!("Model ID: {}", target.model.model_id);
    println!("Form Factor: {}", target.model.form_factor);
    println!("MAC Address: {}", target.address);
    println!("Device Name: {}", target.name);
    println!("Alias: {}", target.alias);
    println!("Connected: {}", target.connected);
    println!("Paired: {}", target.paired);
    println!(
        "Battery: {}",
        target
            .battery_percentage
            .map_or_else(|| "unknown".to_string(), |b| format!("{b}%"))
    );

    println!("\nHardware Capabilities:");
    println!("  ANC / Ambient: {}", target.model.capabilities.has_anc);
    println!("  Low Latency (Game Mode): {}", target.model.capabilities.has_game_mode);
    println!("  Dual-Device Multipoint: {}", target.model.capabilities.has_multipoint);
    println!("  LDAC High-Res: {}", target.model.capabilities.has_ldac);
    println!("  Spatial Audio: {}", target.model.capabilities.has_spatial_audio);
    println!("  Sleep Mode: {}", target.model.capabilities.has_sleep_mode);
    println!("  In-Ear Wearing Detection: {}", target.model.capabilities.has_wearing_detection);
    println!("  10-Band Custom EQ: {}", target.model.capabilities.has_custom_eq);
    println!("  Prompt Volume Adjustment: {}", target.model.capabilities.has_prompt_volume);
    println!("  Auto Power-Off: {}", target.model.capabilities.has_auto_power_off);

    if !target.model.capabilities.supported_anc_scenes.is_empty() {
        let scenes: Vec<_> = target
            .model
            .capabilities
            .supported_anc_scenes
            .iter()
            .map(|s| s.label())
            .collect();
        println!("  Supported ANC Scenes: {}", scenes.join(", "));
    }

    println!("\nGATT Endpoints:");
    println!("  BlueZ Object: {}", target.object_path.as_str());
    println!(
        "  Command Char: {}",
        target
            .command_char_path
            .as_ref()
            .map_or("none", |p| p.as_str())
    );
    println!(
        "  Notify Char: {}",
        target
            .notify_char_path
            .as_ref()
            .map_or("none", |p| p.as_str())
    );

    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_anc(args: AncArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_anc {
        bail!(
            "ANC is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let anc_mode = match args.mode {
        CliAncMode::Off => openhid_qcy::protocol::commands::AncMode::Off,
        CliAncMode::Indoor => openhid_qcy::protocol::commands::AncMode::Indoor,
        CliAncMode::Commuting => openhid_qcy::protocol::commands::AncMode::Commuting,
        CliAncMode::Noisy => openhid_qcy::protocol::commands::AncMode::Noisy,
        CliAncMode::AntiWind => openhid_qcy::protocol::commands::AncMode::AntiWind,
        CliAncMode::Adaptive => openhid_qcy::protocol::commands::AncMode::Adaptive,
        CliAncMode::Transparency => openhid_qcy::protocol::commands::AncMode::Transparency,
    };

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_anc(anc_mode)
        .await
        .context("failed to write ANC command to QCY characteristic")?;

    println!(
        "Successfully set ANC mode to '{:?}' on {} ({})",
        args.mode, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_game_mode(args: GameModeArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_game_mode {
        bail!(
            "Game mode is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_game_mode(args.enabled)
        .await
        .context("failed to write Game Mode command to QCY characteristic")?;

    let state_str = if args.enabled { "enabled" } else { "disabled" };
    println!(
        "Successfully {} Game Mode on {} ({})",
        state_str, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_multipoint(args: ToggleArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_multipoint {
        bail!(
            "Multipoint is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_multipoint(args.enabled)
        .await
        .context("failed to write Multipoint command to QCY characteristic")?;

    let state_str = if args.enabled { "enabled" } else { "disabled" };
    println!(
        "Successfully {} Multipoint on {} ({})",
        state_str, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_ldac(args: ToggleArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_ldac {
        bail!(
            "LDAC is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_ldac(args.enabled)
        .await
        .context("failed to write LDAC command to QCY characteristic")?;

    let state_str = if args.enabled { "enabled" } else { "disabled" };
    println!(
        "Successfully {} LDAC on {} ({})",
        state_str, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_prompt_volume(args: PromptVolumeArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_prompt_volume {
        bail!(
            "Prompt volume adjustment is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_prompt_volume(args.volume)
        .await
        .context("failed to write prompt volume command")?;

    println!(
        "Successfully set Prompt Volume to {} on {} ({})",
        args.volume, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_auto_off(args: AutoOffArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    if !target.model.capabilities.has_auto_power_off {
        bail!(
            "Auto power-off timer is not supported on {} ({})",
            target.model.marketing_name, target.address
        );
    }

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_auto_power_off(args.minutes)
        .await
        .context("failed to write auto power-off command")?;

    println!(
        "Successfully set Auto Power-Off to {} min on {} ({})",
        args.minutes, target.model.marketing_name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
fn find_target_device<'a>(
    devices: &'a [openhid_qcy::transport::DiscoveredQcyDevice],
    address_filter: Option<&str>,
) -> Result<&'a openhid_qcy::transport::DiscoveredQcyDevice> {
    if devices.is_empty() {
        bail!("No QCY devices found in BlueZ. Make sure your device is paired.");
    }

    if let Some(addr) = address_filter {
        devices
            .iter()
            .find(|d| d.address.eq_ignore_ascii_case(addr))
            .ok_or_else(|| anyhow::anyhow!("No QCY device found with address '{addr}'"))
    } else {
        // Prefer connected device, or fallback to first device
        let connected: Vec<_> = devices.iter().filter(|d| d.connected).collect();
        if connected.len() == 1 {
            Ok(connected[0])
        } else if connected.len() > 1 {
            bail!(
                "Multiple QCY devices are connected. Please specify --address <MAC> (available: {})",
                connected.iter().map(|d| d.address.as_str()).collect::<Vec<_>>().join(", ")
            );
        } else {
            // None connected
            if devices.len() == 1 {
                Ok(&devices[0])
            } else {
                bail!(
                    "No QCY device is currently connected. Please connect or specify --address <MAC> (available: {})",
                    devices.iter().map(|d| d.address.as_str()).collect::<Vec<_>>().join(", ")
                );
            }
        }
    }
}

use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub enum QcyCmd {
    /// List discovered QCY Bluetooth devices known to BlueZ.
    List,
    /// Set Active Noise Cancellation / Transparency mode.
    Anc(AncArgs),
    /// Turn Low Latency (Game Mode) on or off.
    GameMode(GameModeArgs),
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
                Self::Anc(args) => run_anc(args).await,
                Self::GameMode(args) => run_game_mode(args).await,
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

        println!();
        println!("{} [{}]", dev.name, dev.address);
        println!("  Alias: {}", dev.alias);
        println!("  State: {conn_status}, {paired_status}");
        println!("  Battery: {battery}");
        println!("  GATT command ready: {has_cmd}");
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_anc(args: AncArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

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
        args.mode, target.name, target.address
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_game_mode(args: GameModeArgs) -> Result<ExitCode> {
    let devices = openhid_qcy::transport::enumerate_bluez_qcy_devices()
        .await
        .context("failed to enumerate QCY devices")?;

    let target = find_target_device(&devices, args.address.as_deref())?;

    let client = openhid_qcy::transport::BluezQcyClient::new(target.clone());
    client
        .set_game_mode(args.enabled)
        .await
        .context("failed to write Game Mode command to QCY characteristic")?;

    let state_str = if args.enabled { "enabled" } else { "disabled" };
    println!(
        "Successfully {} Game Mode on {} ({})",
        state_str, target.name, target.address
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

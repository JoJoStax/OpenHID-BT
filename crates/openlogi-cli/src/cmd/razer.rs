use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub enum RazerCmd {
    /// List discovered Razer Bluetooth devices with detected model & capabilities.
    List,
    /// Inspect details and capabilities of a Razer Bluetooth device.
    Info(DeviceFilterArgs),
    /// Set Chroma BLE LED lighting mode (off, on, spectrum, static, breathing, audio-meter).
    Led(LedArgs),
    /// Set auto-standby inactivity timer.
    Standby(StandbyArgs),
    /// Send a raw hex payload to the Razer BLE command characteristic.
    Raw(RawArgs),
}

#[derive(Debug, Args)]
pub struct DeviceFilterArgs {
    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliLedMode {
    Off,
    On,
    Spectrum,
    Static,
    Breathing,
    #[value(name = "audio-meter", alias = "audiometer")]
    AudioMeter,
}

#[derive(Debug, Args)]
pub struct LedArgs {
    /// LED lighting mode.
    #[arg(value_enum)]
    pub mode: CliLedMode,

    /// Primary hex RGB color (e.g. 44d62c or #ff0000) for static, breathing, or audio-meter.
    #[arg(short, long, default_value = "44d62c")]
    pub color: String,

    /// Secondary hex RGB color for breathing mode.
    #[arg(short = '2', long, default_value = "0000ff")]
    pub secondary: String,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliAutoStandby {
    Never,
    #[value(name = "5m")]
    Minutes5,
    #[value(name = "15m")]
    Minutes15,
    #[value(name = "30m")]
    Minutes30,
    #[value(name = "45m")]
    Minutes45,
    #[value(name = "60m")]
    Minutes60,
}

#[derive(Debug, Args)]
pub struct StandbyArgs {
    /// Auto-standby timeout duration.
    #[arg(value_enum)]
    pub duration: CliAutoStandby,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct RawArgs {
    /// Hex payload bytes (e.g. "c30003ff0000" or "c3 00 03 ff 00 00").
    pub hex_bytes: String,

    /// Bluetooth MAC address of the target device (optional if only one device is connected).
    #[arg(short, long)]
    pub address: Option<String>,
}

impl RazerCmd {
    pub async fn run(self) -> Result<ExitCode> {
        #[cfg(not(target_os = "linux"))]
        {
            bail!("Razer Bluetooth support is currently only available on Linux via BlueZ.");
        }

        #[cfg(target_os = "linux")]
        {
            match self {
                Self::List => run_list().await,
                Self::Info(args) => run_info(args).await,
                Self::Led(args) => run_led(args).await,
                Self::Standby(args) => run_standby(args).await,
                Self::Raw(args) => run_raw(args).await,
            }
        }
    }
}

#[cfg(target_os = "linux")]
async fn resolve_target(
    address: Option<&str>,
) -> Result<openhid_razer::transport::DiscoveredRazerDevice> {
    let devices = openhid_razer::transport::enumerate_bluez_razer_devices()
        .await
        .context("failed to query BlueZ D-Bus for Razer devices")?;

    if devices.is_empty() {
        bail!("No Razer Bluetooth devices found in BlueZ.");
    }

    if let Some(addr) = address {
        devices
            .into_iter()
            .find(|d| d.address.eq_ignore_ascii_case(addr))
            .with_context(|| format!("No Razer device found with MAC address {addr}"))
    } else {
        let connected: Vec<_> = devices.iter().filter(|d| d.connected).cloned().collect();
        match connected.len() {
            1 => Ok(connected.into_iter().next().context("missing device")?),
            0 if devices.len() == 1 => Ok(devices.into_iter().next().context("missing device")?),
            0 => bail!(
                "No Razer devices are currently connected. Specify --address to target a paired device."
            ),
            _ => bail!("Multiple Razer devices are connected. Specify --address to choose one."),
        }
    }
}

#[cfg(target_os = "linux")]
async fn run_list() -> Result<ExitCode> {
    let devices = openhid_razer::transport::enumerate_bluez_razer_devices()
        .await
        .context("failed to query BlueZ D-Bus for Razer devices")?;

    if devices.is_empty() {
        println!("No Razer Bluetooth devices known to BlueZ.");
        return Ok(ExitCode::from(2));
    }

    for dev in devices {
        let status = if dev.connected { "Connected" } else { "Offline" };
        let battery = dev
            .battery_percentage
            .map_or_else(|| "N/A".to_string(), |b| format!("{b}%"));
        let gatt = if dev.command_char_path.is_some() {
            "GATT Ready"
        } else {
            "No GATT"
        };
        println!(
            "{}  [{}]  {} ({})  Bat: {}  {}",
            dev.address,
            status,
            dev.model.marketing_name,
            dev.model.form_factor,
            battery,
            gatt
        );
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_info(args: DeviceFilterArgs) -> Result<ExitCode> {
    let dev = resolve_target(args.address.as_deref()).await?;

    println!("Device:       {}", dev.model.marketing_name);
    println!("Model ID:     {}", dev.model.model_id);
    println!("Form Factor:  {}", dev.model.form_factor);
    println!("MAC Address:  {}", dev.address);
    println!("BlueZ Name:   {}", dev.name);
    println!("Connected:    {}", dev.connected);
    println!("Paired:       {}", dev.paired);
    if let Some(bat) = dev.battery_percentage {
        println!("Battery:      {bat}%");
    }
    println!(
        "GATT Command: {}",
        dev.command_char_path
            .as_ref()
            .map_or("Not resolved", |p| p.as_str())
    );
    println!();
    println!("Capabilities:");
    println!("  Chroma BLE:   {}", dev.model.capabilities.has_chroma_ble);
    println!("  Auto Standby: {}", dev.model.capabilities.has_auto_standby);
    println!("  ANC:          {}", dev.model.capabilities.has_anc);
    println!("  Game Mode:    {}", dev.model.capabilities.has_game_mode);
    println!("  EQ:           {}", dev.model.capabilities.has_eq);

    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_led(args: LedArgs) -> Result<ExitCode> {
    use openhid_razer::protocol::commands::{RazerLedMode, RgbColor};

    let primary = RgbColor::from_hex(&args.color)
        .with_context(|| format!("invalid primary hex color '{}'", args.color))?;
    let secondary = RgbColor::from_hex(&args.secondary)
        .with_context(|| format!("invalid secondary hex color '{}'", args.secondary))?;

    let mode = match args.mode {
        CliLedMode::Off => RazerLedMode::Off,
        CliLedMode::On => RazerLedMode::On,
        CliLedMode::Spectrum => RazerLedMode::Spectrum,
        CliLedMode::Static => RazerLedMode::Static(primary),
        CliLedMode::Breathing => RazerLedMode::Breathing { primary, secondary },
        CliLedMode::AudioMeter => RazerLedMode::AudioMeter(primary),
    };

    let dev = resolve_target(args.address.as_deref()).await?;
    let client = openhid_razer::transport::BluezRazerClient::new(dev);
    client
        .set_led_mode(mode)
        .await
        .context("failed to send Razer Chroma BLE LED command")?;

    println!("Set LED mode on {} to {:?}", client.device.address, mode);
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_standby(args: StandbyArgs) -> Result<ExitCode> {
    use openhid_razer::protocol::commands::AutoStandby;

    let duration = match args.duration {
        CliAutoStandby::Never => AutoStandby::Never,
        CliAutoStandby::Minutes5 => AutoStandby::Minutes5,
        CliAutoStandby::Minutes15 => AutoStandby::Minutes15,
        CliAutoStandby::Minutes30 => AutoStandby::Minutes30,
        CliAutoStandby::Minutes45 => AutoStandby::Minutes45,
        CliAutoStandby::Minutes60 => AutoStandby::Minutes60,
    };

    let dev = resolve_target(args.address.as_deref()).await?;
    let client = openhid_razer::transport::BluezRazerClient::new(dev);
    client
        .set_auto_standby(duration)
        .await
        .context("failed to set Razer auto-standby timeout")?;

    println!(
        "Set auto-standby on {} to {}",
        client.device.address,
        duration.label()
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(target_os = "linux")]
async fn run_raw(args: RawArgs) -> Result<ExitCode> {
    let payload = super::bt::parse_hex_bytes(&args.hex_bytes)?;
    let dev = resolve_target(args.address.as_deref()).await?;
    let client = openhid_razer::transport::BluezRazerClient::new(dev);
    client
        .write_raw(&payload)
        .await
        .context("failed to write raw Razer BLE command")?;

    println!(
        "Wrote {} bytes to {}: {}",
        payload.len(),
        client.device.address,
        super::bt::format_hex_ascii(&payload)
    );
    Ok(ExitCode::SUCCESS)
}

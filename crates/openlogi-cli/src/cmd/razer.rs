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
    /// Manage Razer USB and 2.4GHz HyperSpeed mice.
    #[command(subcommand)]
    Mouse(MouseCmd),
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
        if let Self::Mouse(cmd) = self {
            return cmd.run().await;
        }

        #[cfg(not(target_os = "linux"))]
        {
            bail!("Razer Bluetooth headset support is currently only available on Linux via BlueZ.");
        }

        #[cfg(target_os = "linux")]
        {
            match self {
                Self::List => run_list().await,
                Self::Info(args) => run_info(args).await,
                Self::Led(args) => run_led(args).await,
                Self::Standby(args) => run_standby(args).await,
                Self::Raw(args) => run_raw(args).await,
                Self::Mouse(_) => unreachable!(),
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

#[derive(Debug, Subcommand)]
pub enum MouseCmd {
    /// List connected Razer USB / 2.4GHz HyperSpeed mice.
    List,
    /// Show firmware, serial, DPI, polling rate, and battery status of a Razer mouse.
    Info(MouseTargetArgs),
    /// Get or set mouse DPI (e.g. `openhid razer mouse dpi 1600` or `dpi 1600 1200`).
    Dpi(MouseDpiArgs),
    /// Get or set polling rate in Hz (125, 500, 1000, 2000, 4000, 8000).
    Poll(MousePollArgs),
    /// Set Chroma RGB lighting on a Razer mouse.
    Led(MouseLedArgs),
    /// Get or set SmartReel scroll wheel mode (tactile or free-spin).
    Scroll(MouseScrollArgs),
    /// Configure wireless power settings (idle timeout and low-battery threshold).
    Power(MousePowerArgs),
    /// Send a raw 90-byte Razer HID feature report (for protocol testing).
    Raw(MouseRawArgs),
}

#[derive(Debug, Args)]
pub struct MouseTargetArgs {
    /// USB Product ID in hex (e.g. 00c1) or serial substring when multiple mice are connected.
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Args)]
pub struct MouseDpiArgs {
    /// X DPI (50..=35000). Omit to read current DPI.
    pub dpi_x: Option<u16>,
    /// Optional Y DPI (defaults to X DPI).
    pub dpi_y: Option<u16>,
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Args)]
pub struct MousePollArgs {
    /// Polling rate in Hz (125, 500, 1000, 2000, 4000, 8000). Omit to read current rate.
    pub hz: Option<u16>,
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliMouseLedMode {
    Off,
    Static,
    Breathing,
    Spectrum,
    Reactive,
}

#[derive(Debug, Args)]
pub struct MouseLedArgs {
    #[arg(value_enum)]
    pub mode: CliMouseLedMode,
    /// Hex RGB color (e.g. 44d62c).
    #[arg(short, long, default_value = "44d62c")]
    pub color: String,
    /// LED zone ID (0 = all, 1 = scroll wheel, 4 = logo).
    #[arg(short, long, default_value_t = 0)]
    pub zone: u8,
    /// Optional brightness (0..=255).
    #[arg(short, long)]
    pub brightness: Option<u8>,
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CliScrollMode {
    Tactile,
    #[value(name = "free", alias = "free-spin")]
    FreeSpin,
}

#[derive(Debug, Args)]
pub struct MouseScrollArgs {
    /// Scroll mode (`tactile` or `free`). Omit to read current mode.
    #[arg(value_enum)]
    pub mode: Option<CliScrollMode>,
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Args)]
pub struct MousePowerArgs {
    /// Idle sleep timeout in seconds (60..=900).
    #[arg(long)]
    pub idle: Option<u16>,
    /// Low-battery warning threshold percentage (5..=100).
    #[arg(long)]
    pub low_battery: Option<u8>,
    #[arg(short, long)]
    pub device: Option<String>,
}

#[derive(Debug, Args)]
pub struct MouseRawArgs {
    /// Command class byte in hex (e.g. 04).
    pub command_class: String,
    /// Command ID byte in hex (e.g. 85).
    pub command_id: String,
    /// Optional hex argument bytes.
    pub args_hex: Option<String>,
    /// Optional transaction ID in hex (default: device's transaction ID).
    #[arg(short, long)]
    pub tx: Option<String>,
    #[arg(short, long)]
    pub device: Option<String>,
}

impl MouseCmd {
    pub async fn run(self) -> Result<ExitCode> {
        match self {
            Self::List => run_mouse_list().await,
            Self::Info(args) => run_mouse_info(args).await,
            Self::Dpi(args) => run_mouse_dpi(args).await,
            Self::Poll(args) => run_mouse_poll(args).await,
            Self::Led(args) => run_mouse_led(args).await,
            Self::Scroll(args) => run_mouse_scroll(args).await,
            Self::Power(args) => run_mouse_power(args).await,
            Self::Raw(args) => run_mouse_raw(args).await,
        }
    }
}

async fn resolve_mouse_target(
    filter: Option<&str>,
) -> Result<openhid_razer::mouse::DiscoveredRazerMouse> {
    let mice = openhid_razer::mouse::enumerate_razer_mice()
        .await
        .context("failed to enumerate Razer USB HID mice")?;

    if mice.is_empty() {
        bail!("No Razer USB/2.4GHz mice found.");
    }

    if let Some(f) = filter {
        let f_lower = f.trim_start_matches("0x").to_ascii_lowercase();
        mice.into_iter()
            .find(|m| {
                format!("{:04x}", m.product_id) == f_lower
                    || m.model.model_id.to_ascii_lowercase().contains(&f_lower)
                    || m.serial_number
                        .as_deref()
                        .is_some_and(|s| s.to_ascii_lowercase().contains(&f_lower))
            })
            .with_context(|| format!("No Razer mouse matching '{f}'"))
    } else if mice.len() == 1 {
        Ok(mice.into_iter().next().expect("len == 1"))
    } else {
        let choices: Vec<String> = mice
            .iter()
            .map(|m| format!("{} (0x{:04x})", m.model.marketing_name, m.product_id))
            .collect();
        bail!(
            "Multiple Razer mice connected ({}). Specify one with --device <PID|SERIAL>.",
            choices.join(", ")
        );
    }
}

async fn run_mouse_list() -> Result<ExitCode> {
    let mice = openhid_razer::mouse::enumerate_razer_mice()
        .await
        .context("failed to enumerate Razer USB HID mice")?;

    if mice.is_empty() {
        println!("No Razer USB/2.4GHz mice found.");
        return Ok(ExitCode::SUCCESS);
    }

    for mouse in mice {
        let caps = mouse.model.capabilities;
        println!(
            "{} [1532:{:04x}] ({}) — max {} DPI, wireless={}, hyperpolling={}, chroma={}, smart_reel={}",
            mouse.model.marketing_name,
            mouse.product_id,
            mouse.model.model_id,
            caps.max_dpi,
            caps.is_wireless,
            caps.supports_hyperpolling,
            caps.has_chroma,
            caps.has_smart_reel
        );
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_info(args: MouseTargetArgs) -> Result<ExitCode> {
    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    println!(
        "{} [1532:{:04x}]",
        client.device.model.marketing_name, client.device.product_id
    );
    if let Ok((major, minor)) = client.get_firmware_version().await {
        println!("  Firmware:     v{major}.{minor:02}");
    }
    if let Ok(serial) = client.get_serial().await {
        println!("  Serial:       {serial}");
    }
    if let Ok((x, y)) = client.get_dpi().await {
        println!("  DPI:          {x} x {y}");
    }
    if let Ok(rate) = client.get_polling_rate().await {
        println!("  Polling Rate: {} Hz", rate as u16);
    }
    if client.device.model.capabilities.is_wireless
        && let Ok(pct) = client.get_battery().await
    {
        let charging = client.is_charging().await.unwrap_or(false);
        println!(
            "  Battery:      {pct}%{}",
            if charging { " (charging)" } else { "" }
        );
    }
    if client.device.model.capabilities.has_smart_reel
        && let Ok(mode) = client.get_scroll_mode().await
    {
        println!("  Scroll Mode:  {mode:?}");
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_dpi(args: MouseDpiArgs) -> Result<ExitCode> {
    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    if let Some(x) = args.dpi_x {
        let y = args.dpi_y.unwrap_or(x);
        client
            .set_dpi(x, y)
            .await
            .context("failed to set Razer mouse DPI")?;
        println!("Set DPI on {} to {x} x {y}", client.device.model.marketing_name);
    } else {
        let (x, y) = client
            .get_dpi()
            .await
            .context("failed to read Razer mouse DPI")?;
        println!("Current DPI: {x} x {y}");
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_poll(args: MousePollArgs) -> Result<ExitCode> {
    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    if let Some(hz) = args.hz {
        let rate = openhid_razer::mouse::PollingRate::from_hz(hz)?;
        client
            .set_polling_rate(rate)
            .await
            .context("failed to set Razer mouse polling rate")?;
        println!(
            "Set polling rate on {} to {hz} Hz",
            client.device.model.marketing_name
        );
    } else {
        let rate = client
            .get_polling_rate()
            .await
            .context("failed to read Razer mouse polling rate")?;
        println!("Current polling rate: {} Hz", rate as u16);
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_led(args: MouseLedArgs) -> Result<ExitCode> {
    use openhid_razer::mouse::MouseLedEffect;

    let [r, g, b] = parse_rgb_hex(&args.color)?;
    let effect = match args.mode {
        CliMouseLedMode::Off => MouseLedEffect::Off,
        CliMouseLedMode::Static => MouseLedEffect::Static { r, g, b },
        CliMouseLedMode::Breathing => MouseLedEffect::Breathing { r, g, b },
        CliMouseLedMode::Spectrum => MouseLedEffect::Spectrum,
        CliMouseLedMode::Reactive => MouseLedEffect::Reactive {
            speed: 2,
            r,
            g,
            b,
        },
    };

    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    if let Some(brightness) = args.brightness {
        client
            .set_brightness(args.zone, brightness)
            .await
            .context("failed to set mouse LED brightness")?;
    }
    client
        .set_led(args.zone, effect)
        .await
        .context("failed to set mouse LED effect")?;

    println!(
        "Updated Chroma LED (zone {}) on {} to {:?}",
        args.zone, client.device.model.marketing_name, effect
    );
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_scroll(args: MouseScrollArgs) -> Result<ExitCode> {
    use openhid_razer::mouse::ScrollMode;

    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    if let Some(m) = args.mode {
        let mode = match m {
            CliScrollMode::Tactile => ScrollMode::Tactile,
            CliScrollMode::FreeSpin => ScrollMode::FreeSpin,
        };
        client
            .set_scroll_mode(mode)
            .await
            .context("failed to set Razer mouse scroll mode")?;
        println!(
            "Set scroll mode on {} to {mode:?}",
            client.device.model.marketing_name
        );
    } else {
        let mode = client
            .get_scroll_mode()
            .await
            .context("failed to read Razer mouse scroll mode")?;
        println!("Current scroll mode: {mode:?}");
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_power(args: MousePowerArgs) -> Result<ExitCode> {
    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);

    if args.idle.is_none() && args.low_battery.is_none() {
        let pct = client
            .get_battery()
            .await
            .context("failed to read battery level")?;
        let charging = client.is_charging().await.unwrap_or(false);
        println!(
            "Battery: {pct}%{}",
            if charging { " (charging)" } else { "" }
        );
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(secs) = args.idle {
        client
            .set_idle_time(secs)
            .await
            .context("failed to set idle timeout")?;
        println!("Set idle timeout to {}s", secs.clamp(60, 900));
    }
    if let Some(pct) = args.low_battery {
        client
            .set_low_battery_threshold(pct)
            .await
            .context("failed to set low-battery threshold")?;
        println!("Set low-battery threshold to {}%", pct.clamp(5, 100));
    }
    Ok(ExitCode::SUCCESS)
}

async fn run_mouse_raw(args: MouseRawArgs) -> Result<ExitCode> {
    let class = u8::from_str_radix(args.command_class.trim_start_matches("0x"), 16)
        .context("invalid hex command class")?;
    let id = u8::from_str_radix(args.command_id.trim_start_matches("0x"), 16)
        .context("invalid hex command id")?;
    let arg_bytes = match args.args_hex {
        Some(h) => super::bt::parse_hex_bytes(&h)?,
        None => Vec::new(),
    };

    let dev = resolve_mouse_target(args.device.as_deref()).await?;
    let tx = match args.tx {
        Some(t) => u8::from_str_radix(t.trim_start_matches("0x"), 16)
            .context("invalid hex transaction id")?,
        None => dev.model.capabilities.transaction_id,
    };

    let req = openhid_razer::mouse::RazerReport::new(tx, class, id, arg_bytes.len() as u8)
        .with_args(&arg_bytes)?;
    let client = openhid_razer::mouse::RazerMouseClient::new(dev);
    let resp = client.send_report(&req).await?;

    println!(
        "Response status=0x{:02x} len={} args={}",
        resp.status,
        resp.data_size,
        super::bt::format_hex_ascii(&resp.arguments[..usize::from(resp.data_size).min(80)])
    );
    Ok(ExitCode::SUCCESS)
}

fn parse_rgb_hex(hex: &str) -> Result<[u8; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        bail!("Invalid hex RGB color '{hex}' (expected 6 hex chars like 44d62c)");
    }
    let r = u8::from_str_radix(&clean[0..2], 16).context("invalid red hex")?;
    let g = u8::from_str_radix(&clean[2..4], 16).context("invalid green hex")?;
    let b = u8::from_str_radix(&clean[4..6], 16).context("invalid blue hex")?;
    Ok([r, g, b])
}

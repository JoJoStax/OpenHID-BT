<h1 align="center">OpenHID/BT</h1>
<p align="center"><strong>⚡️ Universal, local-first HID and Bluetooth peripheral management, written in Rust 🦀<br/>Combining multi-vendor hardware support into a fast, desktop-integrated native experience</strong></p>

<div align="center">
    <a href="https://github.com/JoJoStax/OpenHID-BT/releases" target="_blank">
    <img alt="GitHub downloads" src="https://img.shields.io/github/downloads/JoJoStax/OpenHID-BT/total.svg?style=flat"></a>
    <a href="https://github.com/JoJoStax/OpenHID-BT/commits" target="_blank">
    <img alt="GitHub commit" src="https://img.shields.io/github/commit-activity/m/JoJoStax/OpenHID-BT?style=flat"></a>
    <img alt="Zero Monetization" src="https://img.shields.io/badge/Monetization-None%20(Forever%20Free)-success">
    <img alt="Donations" src="https://img.shields.io/badge/Donations-No%20Donations%20Accepted-blue">
</div>

---

### 🌟 Project Vision & Upstream Foundation

**OpenHID/BT** is a community-driven project built upon the incredible foundation of **[OpenLogi](https://github.com/AprilNEA/OpenLogi)** created by **[@AprilNEA](https://github.com/AprilNEA)**.

- **Built on OpenLogi**: We use OpenLogi as our core base. As OpenLogi develops new features and improvements, we regularly pull and integrate upstream changes to keep our Logitech support top-tier.
- **Expanding Beyond Logitech**: While OpenLogi focuses primarily on Logitech hardware, OpenHID/BT is structured to expand into a unified backend supporting peripherals from multiple manufacturers and open-source projects.
- **QCY Bluetooth Support (Pure Rust Backend)**: Integrated native Bluetooth support for QCY audio devices (H3, H3S, MeloBuds Pro, etc.) based on protocol reverse engineering from [HttpKiwi/OpenQCY](https://github.com/HttpKiwi/OpenQCY) and [GiorgioRafael/OpenQCY-Desktop](https://github.com/GiorgioRafael/OpenQCY-Desktop). Implemented in 100% pure Rust over Linux BlueZ D-Bus (`crates/openhid-qcy`), providing ANC mode switching (Off, Indoor, Commuting, Noisy, Anti-Wind, Adaptive, Transparency) and Low Latency Game Mode directly within the native OpenHID/BT architecture.
- **Razer Bluetooth Headset Support (Experimental)**: Added pure Rust BlueZ GATT support (`crates/openhid-razer`) for Razer Bluetooth headsets (Kraken BT / Kitty Edition, Barracuda, Opus, Hammerhead, BlackShark, Kaira), including Chroma BLE lighting modes and Auto-Standby timers.
- **Bluetooth Debugging & GATT Testing (`openhid bt`)**: Built-in BlueZ inspection, GATT service/characteristic dumping, live notification watching, raw hex writes, and privacy-safe JSON snapshots to help bring up more Bluetooth headsets.
- **100% Free & No Monetization**: This project is and will remain strictly free and open-source. **There is zero monetization, no telemetry, no paid tiers, and no donations are accepted.** If you want to show support, please star the project and support the upstream creators!
- **Desktop Environment Adaptability**: OpenHID/BT keeps OpenLogi's clean, modern visual design while adapting dynamically to Linux desktop environments (**KDE Plasma**, **GNOME**, **Hyprland**, etc.), matching your system's color schemes and accent colors automatically.
- **Enhanced Device Support**: Includes fixes for modern gaming hardware such as the **Logitech G502 X Lightspeed** (Host Mode switching to prevent DPI resetting to 1800, `0x8110` Button Spy capture for DPI Up/Down/Shift and Profile Cycle remapping, and dedicated preview artwork).

Runs on Linux, macOS, and Windows.

---

## Key Features

- **Stay light.** Native Rust + GPUI — instant startup, minimal RAM usage.
- **Adaptive Linux Theming.** Automatically matches colors and accents in KDE Plasma, GNOME, and Hyprland.
- **Multi-Device & Vendor Extensible.** Modular architecture ready to integrate multiple peripheral driver families alongside Logitech HID++ and raw HID.
- **Full G502 X Lightspeed Support.** Reliable button remapping for DPI Up, DPI Down, DPI Shift, and Profile Cycle with Host Mode lock.
- **Plain-text config.** Everything is saved in one human-readable TOML file.
- **Powerful CLI & Daemon.** High-performance background agent and command-line diagnostics alongside the GUI.

## Features

- Devices connected over Logi Bolt receivers, Unifying receivers, Bluetooth, or a wired connection, with battery percentage and charge state
- Easy-Switch channels: the name, OS, and connection of the computer paired on each channel, with a rename on devices that allow it (`0x1815`, supported devices)
- Button remapping via the OS input hook: a built-in action catalog plus custom keyboard shortcuts authored in the TOML config, including independent short/long-press actions and hold-until-release chords for push-to-talk¹
- Per-application profile overlays that auto-switch on app focus (macOS + Windows; Linux on X11 / XWayland only)
- Litra lights: power, brightness, and color temperature, with optional auto power that follows camera activity

**Mouse**

- Capture and remap the middle, mode-shift, and thumbwheel buttons (middle everywhere, the rest where the device exposes them)
- Per-direction gesture bindings with live capture on supported buttons: Middle Click, Back/Forward, DPI/ModeShift, the dedicated gesture button, and the haptic panel
  - DPI/ModeShift gestures require device-reported diversion and raw-XY support.
  - Primary clicks and wheel rotation/tilt cannot be newly assigned gestures.
- Actions Ring: a cursor-centred, eight-slot overlay of actions (`ShowActionsRing`), with per-application layouts
- DPI control with presets and Cycle / Set-preset actions (`0x2201`)
- SmartShift wheel: mode toggle, sensitivity, and a permanent-ratchet panel (`0x2111`)
- Per-device native scroll inversion (`0x2121`, supported devices)

**Keyboard**

- Global F-key remapping: the same action catalog as the mouse, plus power-user actions — typed text, key combos, multi-step workflows (macOS + Windows)
- Static RGB lighting (`0x8070` / `0x8080`, supported devices)

**Camera**

- Any Logitech UVC webcam (Brio, StreamCam, the C920 series, …), plug and play
- Live preview that opens the camera only while you watch — leaving it releases the camera entirely and the LED goes off
- Image controls written straight to the UVC hardware — zoom, pan, tilt, focus, exposure, brightness, contrast, saturation, sharpness, gain, backlight compensation, white balance, tint, anti-flicker, and low-light compensation, with auto-mode toggles for focus / exposure / white balance — so changes apply in Meet / Zoom / OBS and every other app using the camera
- One-click profiles: built-in Default / Streaming / Video call plus custom snapshots; settings persist per camera and are written back to the hardware on the next view

¹ Media key actions use D-Bus MPRIS on Linux; a handful of macOS-specific actions have no universal Linux equivalent and are no-ops. Windows maps platform actions to native equivalents where available.

## Install

> [!IMPORTANT]
> Quit **Logi Options+** / **Razer Synapse** first if installed: only one application can own a given HID receiver or device at a time.

### Linux

#### AppImage (Recommended)

Download the latest `OpenHID-BT-*-linux-x86_64.AppImage` (or `aarch64`) from the [latest release](https://github.com/JoJoStax/OpenHID-BT/releases/latest):

```sh
chmod +x OpenHID-BT-*.AppImage
./OpenHID-BT-*.AppImage
```

To install udev rules so your user can access `/dev/hidraw*`, `/dev/uinput`, and `/dev/input/event*` without `sudo`:

```sh
./OpenHID-BT-*.AppImage --udev-rules | sudo tee /etc/udev/rules.d/70-openlogi.rules >/dev/null
sudo udevadm control --reload-rules && sudo udevadm trigger
```

You can also invoke the bundled `openhid` CLI directly through the AppImage:

```sh
./OpenHID-BT-*.AppImage --cli bt list
```

#### Distribution Packages & Installer

Pre-built packages (`.deb`, `.rpm`, `.pkg.tar.zst`, `.flatpak`) and the Linux installer script are available in [releases](https://github.com/JoJoStax/OpenHID-BT/releases/latest) and [`packaging/linux/`](packaging/linux/install.sh).

For NixOS flakes:

```nix
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.openhid-bt = {
    url = "github:JoJoStax/OpenHID-BT";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, openhid-bt, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux"; # or aarch64-linux
      modules = [
        openhid-bt.nixosModules.default
        { programs.openlogi.enable = true; }
      ];
    };
  };
}
```

#### Build from Source (Linux)

```sh
git clone https://github.com/JoJoStax/OpenHID-BT.git
cd OpenHID-BT
cargo run -p openhid-desktop
# or run the CLI directly:
cargo run -p openhid -- list
```

To build a local AppImage yourself (requires `appimagetool` on `PATH`):

```sh
bash packaging/linux/build-appimage.sh
```

See [docs/INSTALL-linux.md](docs/INSTALL-linux.md) for udev rules, distros without systemd, and additional Linux setup notes.

### macOS

Requires macOS 13 or later.

Download the macOS build from the [latest release](https://github.com/JoJoStax/OpenHID-BT/releases/latest) (or build from source with `cargo run -p openhid-desktop`, which requires full Xcode for GPUI Metal shaders).

### Windows

Download the Windows build from the [latest release](https://github.com/JoJoStax/OpenHID-BT/releases/latest) or build from source (`cargo run -p openhid-desktop`). Keep the GUI executable alongside `openhid-agent.exe`, which owns all device I/O. If you hit any issues, please [report them](https://github.com/JoJoStax/OpenHID-BT/issues).

To build from source on any platform, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Usage (CLI)

The `openhid` CLI provides device inventory, diagnostics, and Bluetooth GATT debugging tools:

```sh
openhid list                       # List HID++, standalone, and Bluetooth devices
openhid qcy list                   # List QCY Bluetooth headsets
openhid razer list                 # List Razer Bluetooth headsets
openhid bt list --audio-only       # Inspect all BlueZ audio/vendor devices
openhid bt gatt --read             # Dump & read a headset's GATT services/characteristics
openhid bt watch                   # Stream live GATT notifications as hex + ASCII
openhid bt snapshot --redact-mac   # Capture a privacy-safe JSON device snapshot
```

See [USAGE.md](docs/USAGE.md) for the full CLI reference.

## Configuration

Settings use plain TOML; saves preserve symlinked config files. See
[CONFIGURATION.md](docs/CONFIGURATION.md).

## Developing

See [DEVELOPMENT.md](docs/DEVELOPMENT.md), including [macOS input-hook safety](docs/DEVELOPMENT.md#macos-input-hook-safety).

## Acknowledgments

- **[OpenLogi](https://github.com/AprilNEA/OpenLogi)** by [@AprilNEA](https://github.com/AprilNEA) — the foundational upstream project that makes this work possible
- **Windows, cameras, and i18n** by [@davidbudnick](https://github.com/davidbudnick) — keyboard RGB, Windows support, Logitech webcam support
- **Linux port** by [@cserby](https://github.com/cserby) — Linux support
- [Solaar](https://github.com/pwr-Solaar/Solaar) by [@pwr](https://github.com/pwr) — open-source HID++ implementation
- [Mouser](https://github.com/TomBadash/Mouser) by [@TomBadash](https://github.com/TomBadash) — a local, account-free Options+ replacement

## License

The code in this repository is dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Third-party code

`crates/openlogi-hidpp` is a vendored fork of [`hidpp`](https://crates.io/crates/hidpp)
by [@lus](https://github.com/lus), licensed 0BSD.

### Logo & brand assets

Thanks to [@kubai087](https://github.com/kubai087) for designing the OpenLogi
logo. The OpenLogi logo and app icon (the brand assets under
[`design/`](design/)) are © 2026 AprilNEA, all rights reserved, and are not covered by the MIT/Apache
licenses above; see [`design/LICENSE`](design/LICENSE). Forking the code grants
no right to the OpenLogi name, logo, or icon; please don't use them to represent
your own projects, forks, or distributions without prior written permission.

---

**Not affiliated with Logitech, Razer, or QCY.** "Logitech", "MX Master", "Options+", "Razer", and "Chroma" are trademarks of their respective owners.

## Live Statistics & Repo Activity

<div align="center">
  <a href="https://github.com/JoJoStax/OpenHID-BT/stargazers"><img alt="GitHub Stars" src="https://img.shields.io/github/stars/JoJoStax/OpenHID-BT?style=flat"></a>
  <a href="https://github.com/JoJoStax/OpenHID-BT/network/members"><img alt="GitHub Forks" src="https://img.shields.io/github/forks/JoJoStax/OpenHID-BT?style=flat"></a>
  <a href="https://github.com/JoJoStax/OpenHID-BT/issues"><img alt="GitHub Issues" src="https://img.shields.io/github/issues/JoJoStax/OpenHID-BT?style=flat"></a>
  <a href="https://github.com/JoJoStax/OpenHID-BT/releases"><img alt="Latest Release" src="https://img.shields.io/github/v/release/JoJoStax/OpenHID-BT?include_prereleases&style=flat"></a>
</div>

<br/>

![Repobeats analytics image](https://repobeats.com/JoJoStax/OpenHID-BT "Repobeats analytics image")

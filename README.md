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
- **Expanding Beyond Logitech**: While OpenLogi focuses primarily on Logitech hardware, OpenHID/BT is structured to expand into a unified backend supporting peripherals from multiple manufacturers and open-source projects (Razer, Keychron, Sony, QMK/VIA, standard Bluetooth HID, and more).
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
> Quit **Logi Options+** first: the two applications fight over HID++ access, and only one can own a given receiver at a time.

### macOS

Requires macOS 13 or later.

Download the signed, notarized `.dmg` from the [latest release](https://github.com/AprilNEA/OpenLogi/releases/latest) and drag `OpenLogi.app` to `/Applications`.

Or install via [Homebrew](https://brew.sh):

```sh
brew install --cask openlogi
```

The official Homebrew cask is the default installation path. To explicitly
track the latest GitHub release from `aprilnea/tap` instead:

```sh
brew tap aprilnea/tap
brew install --cask aprilnea/tap/openlogi@latest
```

`openlogi@latest` is maintained by OpenLogi's release workflow and may update
before the official cask autobump lands. Install either `openlogi` or
`openlogi@latest`, not both.

### Linux

Download the installer over HTTPS, inspect it, then run it. Do not pipe it into
a shell:

```sh
curl --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --fail --location --silent --show-error \
  --retry 3 --retry-connrefused \
  --output openlogi-install.sh \
  https://raw.githubusercontent.com/AprilNEA/OpenLogi/master/packaging/linux/install.sh
less openlogi-install.sh
sh openlogi-install.sh
rm openlogi-install.sh
```

The script detects apt, dnf, yum, zypper, rpm, or pacman; selects the exact
`.deb`, `.rpm`, or `.pkg.tar.zst` for the machine; authenticates its detached
signature with OpenLogi's embedded minisign public key; and verifies its entry
in the release `SHA256SUMS` before invoking the package manager with `sudo`.
Install `minisign` through your distribution first. Run the script as your
normal user, not with `sudo`. It installs the latest release by default. Use
`--version`, `--package-manager`, `--no-start`, or `--dry-run` when needed.

Packages are published for both `x86_64`/`amd64` and `arm64`/`aarch64`.
Pre-built packages require GLIBC 2.35 or newer (Ubuntu 22.04 baseline).

Each release also carries a Flatpak bundle, for `amd64` and `arm64`:

```sh
flatpak install --user ./openlogi-*.flatpak
```

It is a one-time install: there is no remote behind it, so `flatpak update` has
nothing to check and a new version means downloading the next bundle.

A Flatpak cannot write to `/etc`, so its udev rules go on the host separately.
They ship inside the application, so this needs no extra download:

```sh
flatpak run --command=cat org.openlogi.OpenLogi /app/share/openlogi/udev/70-openlogi.rules | sudo tee /etc/udev/rules.d/70-openlogi.rules >/dev/null && sudo udevadm control --reload-rules && sudo udevadm trigger
```

Until that runs, no devices are detected at all. Nothing extra is needed for
`/dev/uinput`: the rules create its node at boot with `static_node=uinput`, and
opening it loads the module. The agent is launched by the application itself, so
unlike the packages below there is no user service to enable.

NixOS users can instead import the repository's module, which installs the
package and udev rules and starts the agent with the graphical session:

```nix
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.openlogi = {
    url = "github:AprilNEA/OpenLogi";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, openlogi, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux"; # or aarch64-linux
      modules = [
        openlogi.nixosModules.default
        { programs.openlogi.enable = true; }
      ];
    };
  };
}
```

All Linux packages install udev rules that grant your user access to
`/dev/hidraw*`, `/dev/uinput` and your Logitech mouse's `/dev/input/event*`
node without `sudo`. The installer and NixOS module start the agent
automatically; after a manual package installation, enable it for your user:

```sh
systemctl --user enable --now openlogi-agent.service
```

See [docs/INSTALL-linux.md](docs/INSTALL-linux.md) for fixed-version installs,
complete NixOS options, source installs, and distros without systemd.

### Windows

Signed portable `.zip` archives and per-user `.msi` installers (x86_64 and
arm64) are attached to each release. Both ship the GUI (`OpenLogi.exe`)
together with the background agent (`openlogi-agent.exe`), which owns all
device I/O. Keep the two files side by side when using the portable zip, or
the GUI has nothing to connect to.

Windows support has been validated end-to-end on Windows 11 with real
hardware (a wired keyboard and a Unifying-receiver mouse), including
install, in-place upgrade, and uninstall of the MSI. It is newer than the
macOS build, so if you hit a rough edge please
[report it](https://github.com/AprilNEA/OpenLogi/issues). The agent shows a
system-tray icon (Show Main Window / Quit) so the app stays reachable after
the main window is closed. To disable it on Windows, set
`show_in_menu_bar = false` in the TOML `[app_settings]` block and restart the
agent; the GUI toggle is currently macOS-only.

To build from source, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).


## Usage (CLI)

See [USAGE.md](docs/USAGE.md)

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

**Not affiliated with Logitech.** "Logitech", "MX Master", and "Options+" are trademarks of Logitech International S.A.

## Repo activity

![Repobeats analytics image](https://repobeats.com/AprilNEA/OpenLogi "Repobeats analytics image")

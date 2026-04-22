# IQOS CLI

<div align="center">

**A command-line interface for controlling IQOS devices via Bluetooth Low Energy, built on [V-VX/iqos](https://github.com/V-VX/iqos)**

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg?style=flat-square)](https://github.com/v-vx/iqos_cli)

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Commands](#-commands-reference) • [Contributing](#-contributing)

</div>

---

## Table of Contents

- [Overview](#-overview)
- [Architecture](#-architecture)
- [Features](#-features)
- [Device Compatibility](#-device-compatibility)
- [Prerequisites](#-prerequisites)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Commands Reference](#-commands-reference)
- [Examples](#-examples)
- [Troubleshooting](#-troubleshooting)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

## Overview

IQOS CLI is a Rust-based command-line tool for controlling IQOS devices over Bluetooth Low Energy, built on top of [V-VX/iqos](https://github.com/V-VX/iqos). It supports both an interactive REPL and one-shot command execution, so you can either connect once and work from the `iqos>` prompt or run a single command directly from your shell.

## Architecture

All device protocol logic — BLE framing, capability negotiation, command encoding, response parsing — lives in the [iqos crate (V-VX/iqos)](https://github.com/V-VX/iqos). This repository is a thin CLI layer: it handles device discovery, user interaction, and argument parsing, then delegates every device operation to the crate's high-level API.

## Features

- **Automatic Device Discovery** — Scans and connects to IQOS devices via Bluetooth
- **Interactive Console** — REPL with command history (`iqos>` prompt)
- **One-Shot CLI Commands** — Run device commands directly, for example `iqos --model iluma battery`
- **Saved Device Labels** — Remember a connected device and target it later with `--model <label>`
- **Battery Management** — Real-time battery status
- **Brightness Control** — Set LED brightness (all ILUMA models)
- **Vibration Customization** — Configure vibration for heating, puff-end, etc.
- **FlexPuff** — Enable, disable, or check FlexPuff status (ILUMA i / ILUMA i Prime)
- **FlexBattery** — Performance/Eco mode and pause mode (ILUMA i / ILUMA i Prime)
- **Smart Gesture** — Enable/disable smart gesture recognition (ILUMA / ILUMA Prime / ILUMA i / ILUMA i One / ILUMA i Prime)
- **AutoStart** — Automatic heating start (holder models)
- **Device Lock/Unlock** — Lock and unlock the device
- **Diagnosis** — Puff count, days used, battery voltage
- **Device Status** — Firmware, product number, and voltage snapshot
- **Find My IQOS** — Trigger device vibration for locating

## Device Compatibility

| Feature | ILUMA i | ILUMA i One | ILUMA i Prime | ILUMA | ILUMA ONE | ILUMA Prime |
|---------|:-------:|:-----------:|:-------------:|:-----:|:---------:|:-----------:|
| Battery Status | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Device Info | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Diagnosis | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Find My IQOS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Device Lock/Unlock | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vibration Settings | ✅¹ | ✅ | ✅ | ✅¹ | ✅ | ✅ |
| Brightness | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto Start | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ |
| Smart Gesture | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Flex Puff | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |
| Flex Battery | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |

¹ The `charge` vibration flag is only available on ILUMA and ILUMA i (holder-based models with charge-start support).

## Prerequisites

- **Rust 1.92 or later** — [Install Rust](https://rustup.rs/)
- **Bluetooth adapter** — A working Bluetooth adapter on your system
- **Platform-specific dependencies**:

  **Linux:**
  ```bash
  sudo apt-get install libdbus-1-dev pkg-config
  ```

  **macOS / Windows:** No additional dependencies required.

## Installation

### From Source

```bash
git clone https://github.com/v-vx/iqos_cli.git
cd iqos_cli
cargo build --release
./target/release/iqos_cli
```

### Via Cargo

```bash
cargo install --path .
```

## Quick Start

Examples below use `iqos` as the command name. When running directly from this repository, use `./target/release/iqos_cli` or `cargo run --release --` in its place.

### Interactive Mode

1. Enable Bluetooth on your system
2. Turn on your IQOS device and ensure it's in range
3. Run IQOS CLI:
   ```bash
   iqos
   # or during development
   cargo run --release --
   ```
4. Select your device when prompted:
   ```
   Found IQOS: IQOS3_AABBCC (AA:BB:CC:DD:EE:FF)
   Connect to IQOS3_AABBCC (AA:BB:CC:DD:EE:FF)? [y/N]: y
   ```
5. Use commands in the interactive console:
   ```
   iqos> help
   iqos> battery
   Battery Level: 85%

   iqos> brightness high
   Brightness set to high

   iqos> flexbattery eco
   FlexBattery settings updated
   ```

### One-Shot CLI Mode

Run a single command without opening the REPL:

```bash
iqos --model iluma battery
iqos brightness high --model iluma
iqos vibration heating on --model iluma-i --timeout 5
```

`--model` accepts either a built-in model selector or a saved device label. Global options can be placed before the command, after the command, or between command arguments; the CLI normalizes them before execution.

If you pass only a target option and no subcommand, IQOS CLI connects to that target and then starts interactive mode:

```bash
iqos --model minera
```

This is useful after saving a device label, because it skips the manual "Connect?" prompt and connects directly to the saved device.

## Commands Reference

### CLI Invocation

| Command | Description |
|---------|-------------|
| `iqos` | Scan nearby IQOS devices, ask which one to connect to, then open interactive mode |
| `iqos --help` | Show top-level CLI help |
| `iqos help` | Same as `iqos --help` |
| `iqos -v` / `iqos --version` | Print the IQOS CLI version and exit without scanning |
| `iqos --model <model-or-label>` | Connect to a built-in model selector or saved label, then open interactive mode |
| `iqos --model <model-or-label> <command>` | Connect to the selected target and run one command |
| `iqos <command> --model <model-or-label>` | Same as above; global options may be placed after the command |
| `iqos --timeout <secs> ...` | Override the BLE scan timeout |

Built-in model selectors include `iluma`, `iluma-one`, `iluma-prime`, `iluma-i`, `iluma-i-one`, and `iluma-i-prime`. Saved labels are managed with the `device` command.

`-v` / `--version` takes precedence over other arguments before `--`; it prints the CLI version and exits without scanning or connecting.

### General

| Command | Description |
|---------|-------------|
| `help` | List all available commands |
| `version` | Show the IQOS CLI version |
| `info` | Show device model, serial number, GATT metadata, firmware, product number, and battery voltage |
| `battery` | Show current battery level |
| `diagnosis` | Show puff count, days used, and battery voltage |
| `lock` | Lock the device |
| `unlock` | Unlock the device |
| `findmyiqos` | Vibrate the device until Enter is pressed |
| `exit` / `quit` | Exit the CLI |

### Device Memory

| Command | Description |
|---------|-------------|
| `device list` | List saved device labels and metadata |
| `device save <label>` | Save the current or targeted device under a label |
| `device remove <label>` | Remove a saved device label |

Device memory is stored in `config.toml` under the user config directory. The CLI also remembers the last successfully connected device as the default target. That lets you run commands like `iqos battery` after a device has been remembered once. Use labels when you want a stable name for a specific device:

```bash
iqos --model iluma device save minera
iqos device list
iqos --model minera battery
iqos --model minera
iqos device remove minera
```

### Display & Feedback

| Command | Description | Compatibility |
|---------|-------------|---------------|
| `brightness` | Show current brightness level | All models |
| `brightness <low\|high>` | Set LED brightness | All models |
| `vibration` | Show current vibration settings | All models |
| `vibration <flag> <on\|off> ...` | Set one or more vibration flags | All models |

Vibration flags: `heating`, `starting`, `puffend`, `terminated`, `charge`¹

### Advanced Features

| Command | Description | Compatibility |
|---------|-------------|---------------|
| `flexpuff <enable\|disable\|status>` | Manage FlexPuff | ILUMA i / i Prime |
| `flexbattery` | Show FlexBattery mode and pause state | ILUMA i / i Prime |
| `flexbattery <performance\|eco>` | Set battery mode | ILUMA i / i Prime |
| `flexbattery pause <on\|off>` | Toggle pause mode | ILUMA i / i Prime |
| `smartgesture <enable\|disable>` | Toggle Smart Gesture | ILUMA / ILUMA Prime / ILUMA i / i One / i Prime |
| `autostart <on\|off\|status>` | Show or toggle automatic heating start | Holder models |

## Examples

### Battery & Diagnosis
```
iqos> battery
Battery Level: 85%

iqos> diagnosis
Diagnosis:
  Total puffs:     1234
  Days used:       42
  Battery voltage: 3.87V
```

### Brightness
```
iqos> brightness
Brightness: low

iqos> brightness high
Brightness set to high
```

### Vibration
```
iqos> vibration
VibrationSettings { heating_start: true, starting_to_use: true, puff_end: false, manually_terminated: false, charge_start: None }

iqos> vibration heating on puffend off
Vibration settings updated
```

### FlexBattery
```
iqos> flexbattery
FlexBattery: mode=Eco, pause=Some(false)

iqos> flexbattery performance
FlexBattery settings updated

iqos> flexbattery pause on
FlexBattery settings updated
```

### Device Information
```
iqos> info
Device Information:
  Model:           IlumaI
  Model number:    A123
  Serial number:   XXXXXXXXXXXX
  Manufacturer:    Philip Morris International
  Software rev:    X.X.X
  Product number:  XXXXXXXXXXXX
  Stick firmware:  v1.2.3.24
  Holder product:  XXXXXXXXXXXX
  Holder firmware: v1.2.3.24
  Battery voltage: 3.870V
```

## Troubleshooting

### Device Not Found

- Ensure Bluetooth is enabled
- Make sure the IQOS device is powered on and in range
- Restart the device or the CLI and try again

### Connection Failed

- Ensure no other application (e.g. IQOS app) is connected to the device
- Restart the device's Bluetooth and try again
- On macOS, check Bluetooth permissions for the terminal

### Permission Denied (Linux)

```bash
sudo usermod -a -G bluetooth $USER
# Log out and log back in
```

### Command Not Available

- Check the compatibility table — some features are model-specific
- Run `info` to verify the connected device model

## Development

### Project Structure

```
iqos_cli/
├── src/
│   ├── main.rs              # Entry point and BLE device discovery
│   └── loader/              # CLI interface
│       ├── mod.rs           # Console runner (run_console)
│       ├── parser.rs        # IQOSConsole REPL and command dispatch
│       ├── compat.rs        # Device capability workarounds
│       └── cmds/            # Per-command implementations
├── Cargo.toml
└── README.md
```

### Build Commands

```bash
cargo build            # Debug build
cargo build --release  # Optimized build
cargo test             # Run tests
cargo fmt              # Format code
cargo clippy -- -D warnings  # Lint
cargo check            # Fast type-check without linking
```

### Release Workflow

Releases are created automatically when a semantic version tag is pushed:

```bash
git tag -s v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

The release workflow validates tags like `v1.2.3` or `v1.2.3-rc.1`, verifies that the pushed tag already exists, builds packages for Linux, Windows, and macOS, then creates a GitHub Release with generated notes. Tags with a prerelease suffix, such as `-rc.1`, are published as prereleases.

Release packages contain the executable as `iqos` on Linux/macOS and `iqos.exe` on Windows, so users can place the extracted binary on `PATH` and run the `iqos` command directly.

Generated release notes are grouped by PR labels using `.github/release.yml`. Use clear PR titles and apply one of these labels before merging:

| Label | Release notes section |
|-------|-----------------------|
| `breaking`, `breaking-change`, `semver-major` | Breaking Changes |
| `feature`, `enhancement`, `semver-minor` | Features |
| `bug`, `fix`, `semver-patch` | Fixes |
| `documentation`, `docs` | Documentation |
| `chore`, `refactor`, `dependencies`, `ci`, `tests` | Maintenance |
| `skip-changelog`, `ignore-for-release` | Excluded from release notes |

GitHub Releases are the project changelog. Keep merged PR titles user-facing because they become the release-note entries.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Add tests for new functionality
4. Ensure all tests pass and `cargo clippy` is clean
5. Open a Pull Request

### Reporting Bugs

Open an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- OS, Rust version, and IQOS device model

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with [btleplug](https://github.com/deviceplug/btleplug) for Bluetooth Low Energy support.

---

<div align="center">

**Issues:** [github.com/v-vx/iqos_cli/issues](https://github.com/v-vx/iqos_cli/issues)

</div>

# IQOS CLI

<div align="center">

**A command-line interface for controlling IQOS devices via Bluetooth Low Energy, built on [V-VX/iqos](https://github.com/V-VX/iqos)**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
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

IQOS CLI is a Rust-based command-line tool for controlling IQOS devices over Bluetooth Low Energy, built on top of [V-VX/iqos](https://github.com/V-VX/iqos). It scans for nearby devices, prompts for a connection, then drops into an interactive REPL for device control.

## Architecture

All device protocol logic — BLE framing, capability negotiation, command encoding, response parsing — lives in the [iqos crate (V-VX/iqos)](https://github.com/V-VX/iqos). This repository is a thin CLI layer: it handles device discovery, user interaction, and argument parsing, then delegates every device operation to the crate's high-level API.

## Features

- **Automatic Device Discovery** — Scans and connects to IQOS devices via Bluetooth
- **Interactive Console** — REPL with command history (`iqos>` prompt)
- **Battery Management** — Real-time battery status
- **Brightness Control** — Set LED brightness (all ILUMA models)
- **Vibration Customization** — Configure vibration for heating, puff-end, etc.
- **FlexPuff** — Enable, disable, or check FlexPuff status (ILUMA i series)
- **FlexBattery** — Performance/Eco mode and pause mode (ILUMA i / ILUMA i Prime)
- **Smart Gesture** — Enable/disable smart gesture recognition (ILUMA i series)
- **AutoStart** — Automatic heating start (ILUMA / ILUMA i)
- **Device Lock/Unlock** — Lock and unlock the device
- **Diagnosis** — Puff count, days used, battery voltage
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
| Auto Start | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Smart Gesture | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Flex Puff | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Flex Battery | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |

¹ The `charge` vibration flag is only available on ILUMA and ILUMA i (holder-based models with charge-start support).

## Prerequisites

- **Rust 1.70 or later** — [Install Rust](https://rustup.rs/)
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

1. Enable Bluetooth on your system
2. Turn on your IQOS device and ensure it's in range
3. Run IQOS CLI:
   ```bash
   iqos_cli
   # or during development
   cargo run --release
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

## Commands Reference

### General

| Command | Description |
|---------|-------------|
| `help` | List all available commands |
| `info` | Show device model, serial number, and firmware version |
| `battery` | Show current battery level |
| `diagnosis` | Show puff count, days used, and battery voltage |
| `lock` | Lock the device |
| `unlock` | Unlock the device |
| `findmyiqos` | Vibrate the device until Enter is pressed |
| `exit` / `quit` | Exit the CLI |

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
| `flexpuff <enable\|disable\|status>` | Manage FlexPuff | ILUMA i series |
| `flexbattery` | Show FlexBattery mode and pause state | ILUMA i / i Prime |
| `flexbattery <performance\|eco>` | Set battery mode | ILUMA i / i Prime |
| `flexbattery pause <on\|off>` | Toggle pause mode | ILUMA i / i Prime |
| `smartgesture <enable\|disable>` | Toggle Smart Gesture | ILUMA i series |
| `autostart <on\|off>` | Toggle automatic heating start | ILUMA / ILUMA i |

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
Device Model: IQOS ILUMA i
Serial Number: XXXXXXXXXXXX
Firmware Version: X.X.X
Manufacturer: Philip Morris International
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

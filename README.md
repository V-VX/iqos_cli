# IQOS CLI

<div align="center">

**A powerful command-line interface for controlling IQOS devices via Bluetooth Low Energy**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg?style=flat-square)](https://github.com/yourusername/iqos_cli)

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Commands](#-commands) • [Contributing](#-contributing)

</div>

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Features](#-features)
- [Device Compatibility](#-device-comatibility)
- [Prerequisites](#-prerequisites)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Commands Reference](#-commands-reference)
- [Examples](#-examples)
- [Troubleshooting](#-troubleshooting)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

## 🎯 Overview

IQOS CLI is a comprehensive Rust-based command-line tool that provides full control over IQOS devices through Bluetooth Low Energy (BLE) connections. Built with modern Rust practices and async/await patterns, it offers a robust, maintainable, and efficient way to interact with IQOS ILUMA, ILUMA i, and ILUMA ONE devices.

Whether you're looking to customize vibration patterns, adjust brightness levels, or monitor battery status, IQOS CLI provides an intuitive interface to unlock your device's full potential.

## ✨ Features

- 🔍 **Automatic Device Discovery** - Scans and detects IQOS devices automatically via Bluetooth
- 🎮 **Interactive Console** - User-friendly command-line interface with command history
- 🔋 **Battery Management** - Real-time battery status monitoring and power settings
- 💡 **Brightness Control** - Adjust LED brightness levels (ILUMA models)
- 📳 **Vibration Customization** - Configure vibration patterns for various device events
- ⚡ **FlexPuff Support** - Manage flexible puff settings for personalized experience
- 🔌 **FlexBattery Management** - Advanced battery optimization features
- 🤖 **Smart Gesture Control** - Enable/disable smart gesture recognition
- 🚀 **AutoStart Configuration** - Automatic heating start settings
- 🔒 **Device Locking** - Secure your device with lock/unlock functionality
- 📊 **Device Information** - View detailed device info (model, serial, firmware, etc.)
- 🛡️ **Type-Safe API** - Leverages Rust's type system for safe BLE operations

## 🔌 Device Compatibility

| Feature | **IQOS ILUMA i** | **IQOS ILUMA i One** | **IQOS ILUMA i Prime** | **IQOS ILUMA** | **IQOS ILUMA ONE** | IQOS ILUMA Prime |
|--------|--------|----------|------|-----|-----|-----|
| Battery Status | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |
| Device Info | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |
| Device Lock/Unlock | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ | 
| Vibration Settings | ✅ | ✅ | ❓ | ✅ | ❓ | ❓|
| Brightness control | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |
| Smart Gesture | ✅ |  ✅ | ❓ | ✅ | ❓ | ❓ |
| Auto Start | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |
| Flex Puff | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |
| Flex Battery | ✅ | ✅ | ❓ | ✅ | ❓ | ❓ |

## 📋 Prerequisites

Before installing IQOS CLI, ensure you have the following:

- **Rust 1.70 or later** - [Install Rust](https://rustup.rs/)
- **Bluetooth Adapter** - A working Bluetooth adapter on your system
- **Platform-specific dependencies**:
  
  **macOS:**
  ```bash
  # No additional dependencies required
  ```

  **Linux:**
  ```bash
  sudo apt-get install libdbus-1-dev pkg-config
  ```

  **Windows:**
  ```bash
  # No additional dependencies required
  ```

## 🛠️ Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/iqos_cli.git
cd iqos_cli

# Build the project
cargo build --release

# The binary will be available at target/release/iqos_cli
./target/release/iqos_cli
```

### Install via Cargo

```bash
cargo install --path .
```

## 🚀 Quick Start

1. **Enable Bluetooth** on your system

2. **Turn on your IQOS device** and ensure it's in range

3. **Run IQOS CLI**:
   ```bash
   cargo run --release
   # or if installed
   iqos_cli
   ```

4. **Connect to your device**:
   ```
   Found IQOS: IQOS_DEVICE_NAME (XX:XX:XX:XX:XX:XX)
   Connect to IQOS_DEVICE_NAME (XX:XX:XX:XX:XX:XX)? [y/N]: y
   ```

5. **Start using commands** in the interactive console:
   ```
   iqos> help
   iqos> battery
   iqos> brightness
    
   Brightness Level: low

   iqos> brightness high

   Brightness Level: high

   iqos>
   ```

## 📚 Commands Reference

### General Commands

| Command | Description | Example |
|---------|-------------|---------|
| `help` | Display all available commands | `help` |
| `info` | Show device information (model, serial, firmware) | `info` |
| `battery` | Display current battery status | `battery` |
| `lock` | Lock the device | `lock` |
| `unlock` | Unlock the device | `unlock` |
| `reset` | Reset device to factory settings | `reset` |
| `exit` / `quit` | Exit the CLI | `exit` |

### Display & Feedback

| Command | Description | Example | Compatibility |
|---------|-------------|---------|---------------|
| `brightness <level>` | Set LED brightness (1-10) | `brightness 7` | ILUMA, ILUMA i |
| `vibration` | Configure vibration patterns | `vibration` | All models |

### Advanced Features

| Command | Description | Example | Compatibility |
|---------|-------------|---------|---------------|
| `flexpuff <setting>` | Configure FlexPuff settings | `flexpuff enable` | ILUMA i |
| `flexbattery <setting>` | Configure FlexBattery settings | `flexbattery on` | ILUMA, ILUMA i |
| `smartgesture <on\|off>` | Enable/disable smart gestures | `smartgesture on` | ILUMA i |
| `autostart <on\|off>` | Configure automatic heating start | `autostart on` | ILUMA, ILUMA i |

## 💡 Examples

### Check Battery Status
```bash
iqos> battery
Battery Level: 85%
Charging: No
Status: Ready
```

### Adjust Brightness
```bash
iqos> brightness 8
Brightness set to level 8
```

### Configure Vibration Pattern
```bash
iqos> vibration
Select vibration event:
1. Heating Start
2. Ready to Use
3. Battery Low
Selection: 1
Enter vibration intensity (0-100): 75
Vibration pattern updated successfully
```

### Enable FlexBattery
```bash
iqos> flexbattery on
FlexBattery enabled - Your device will optimize battery usage
```

### View Device Information
```bash
iqos> info
Device Model: IQOS ILUMA
Serial Number: XXXXXXXXXXXX
Firmware Version: X.X.X
Manufacturer: Philip Morris International
```

## 🔧 Troubleshooting

### Device Not Found

**Problem:** CLI doesn't detect your IQOS device

**Solutions:**
- Ensure Bluetooth is enabled on your system
- Make sure your IQOS device is turned on and in range
- Restart your IQOS device
- Try scanning again by restarting the CLI

### Connection Failed

**Problem:** Unable to connect to the device

**Solutions:**
- Ensure no other application is connected to your IQOS
- Check if your device needs to be unpaired and re-paired
- Restart both your computer's Bluetooth and the IQOS device
- Make sure you have proper Bluetooth permissions

### Permission Denied (Linux)

**Problem:** Bluetooth access denied

**Solution:**
```bash
# Add your user to the bluetooth group
sudo usermod -a -G bluetooth $USER
# Log out and log back in
```

### Command Not Available

**Problem:** Specific command doesn't work

**Solutions:**
- Check device compatibility - some features are model-specific
- Ensure your device firmware is up to date
- Verify you're connected to the device with `info` command

## 🛠️ Development

### Project Structure

```
iqos_cli/
├── src/
│   ├── main.rs              # Entry point and device discovery
│   ├── iqos/                # Core IQOS device implementation
│   │   ├── mod.rs           # Module definitions and constants
│   │   ├── builder.rs       # Device builder pattern
│   │   ├── device.rs        # Device trait implementations
│   │   ├── iqos.rs          # Main IQOS BLE interface
│   │   ├── iluma.rs         # ILUMA specific implementation
│   │   ├── iluma_i.rs       # ILUMA i specific implementation
│   │   ├── brightness/      # Brightness control
│   │   ├── vibration/       # Vibration settings
│   │   ├── flexbattery/     # FlexBattery features
│   │   └── flexpuff/        # FlexPuff features
│   └── loader/              # CLI interface
│       ├── mod.rs           # Console runner
│       ├── parser.rs        # Command parser
│       └── cmds/            # Command implementations
├── Cargo.toml               # Project dependencies
└── README.md                # This file
```

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open

# Check code without building
cargo check
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test iqos::tests
```

### Code Style

This project follows Rust's official style guidelines and uses:
- `rustfmt` for code formatting
- `clippy` for linting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
   - Follow the project's coding guidelines
   - Add tests for new features
   - Update documentation as needed
4. **Commit your changes** (`git commit -m 'Add amazing feature'`)
5. **Push to the branch** (`git push origin feature/amazing-feature`)
6. **Open a Pull Request**

### Development Guidelines

- Write clean, idiomatic Rust code
- Follow the existing code structure and patterns
- Add comprehensive tests for new features
- Update documentation for user-facing changes
- Use meaningful commit messages
- Ensure all tests pass before submitting PR

### Reporting Bugs

Found a bug? Please open an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version, device model)
- Relevant logs or error messages

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [btleplug](https://github.com/deviceplug/btleplug) - Rust Bluetooth Low Energy library
- Inspired by the Rust community's commitment to safe and efficient systems programming
- Thanks to all contributors who help improve this project

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/iqos_cli/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/iqos_cli/discussions)

---

<div align="center">

**Made with ❤️ and 🦀 Rust**

⭐ Star this repo if you find it useful!

</div>

use std::time::Duration;

use clap::{Parser, Subcommand};
use iqos::DeviceModel;

#[derive(Debug, Parser)]
#[command(
    name = "iqos",
    version,
    about = "Control IQOS devices over BLE",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Target device model or saved device label.
    #[arg(long, value_name = "target")]
    pub model: Option<String>,

    /// BLE scan timeout in seconds.
    #[arg(long, value_name = "secs")]
    pub timeout: Option<u64>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Configure auto-start.
    Autostart {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
    /// Display battery level.
    Battery,
    /// Set display brightness.
    Brightness {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
    /// Manage saved devices.
    Device {
        #[command(subcommand)]
        command: DeviceCommand,
    },
    /// Retrieve telemetry data.
    Diagnosis,
    /// Activate find-my-device vibration.
    Findmyiqos,
    /// Configure FlexBattery.
    Flexbattery {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
    /// Configure FlexPuff.
    Flexpuff {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
    /// Display command help for the connected device.
    Help,
    /// Device metadata, firmware, and voltage snapshot.
    Info,
    /// Lock the device.
    Lock,
    /// Configure SmartGesture.
    Smartgesture {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
    /// Unlock the device.
    Unlock,
    /// Configure vibration feedback.
    Vibration {
        #[arg(
            value_name = "arg",
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum DeviceCommand {
    /// Save the currently targeted device under a label.
    Save {
        #[arg(value_name = "label")]
        label: String,
    },
    /// List saved device labels.
    List,
    /// Remove a saved device label.
    Remove {
        #[arg(value_name = "label")]
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneShotCommand {
    Registered {
        name: &'static str,
        args: Vec<String>,
    },
    DeviceSave {
        label: String,
    },
    DeviceList,
    DeviceRemove {
        label: String,
    },
}

impl CliCommand {
    pub fn into_one_shot(self) -> OneShotCommand {
        match self {
            Self::Autostart { args } => registered("autostart", args),
            Self::Battery => registered("battery", Vec::new()),
            Self::Brightness { args } => registered("brightness", args),
            Self::Device { command } => match command {
                DeviceCommand::Save { label } => OneShotCommand::DeviceSave { label },
                DeviceCommand::List => OneShotCommand::DeviceList,
                DeviceCommand::Remove { label } => OneShotCommand::DeviceRemove { label },
            },
            Self::Diagnosis => registered("diagnosis", Vec::new()),
            Self::Findmyiqos => registered("findmyiqos", Vec::new()),
            Self::Flexbattery { args } => registered("flexbattery", args),
            Self::Flexpuff { args } => registered("flexpuff", args),
            Self::Help => registered("help", Vec::new()),
            Self::Info => registered("info", Vec::new()),
            Self::Lock => registered("lock", Vec::new()),
            Self::Smartgesture { args } => registered("smartgesture", args),
            Self::Unlock => registered("unlock", Vec::new()),
            Self::Vibration { args } => registered("vibration", args),
        }
    }
}

fn registered(name: &'static str, mut args: Vec<String>) -> OneShotCommand {
    args.insert(0, name.to_string());
    OneShotCommand::Registered { name, args }
}

pub fn parse_device_model(value: &str) -> Option<DeviceModel> {
    let normalized = value.trim().to_ascii_lowercase().replace(['_', ' '], "-");

    match normalized.as_str() {
        "iluma" => Some(DeviceModel::Iluma),
        "iluma-prime" => Some(DeviceModel::IlumaPrime),
        "iluma-one" => Some(DeviceModel::IlumaOne),
        "iluma-i" => Some(DeviceModel::IlumaI),
        "iluma-i-prime" => Some(DeviceModel::IlumaIPrime),
        "iluma-i-one" => Some(DeviceModel::IlumaIOne),
        _ => None,
    }
}

pub fn scan_timeout(cli_value: Option<u64>) -> Duration {
    let seconds = cli_value
        .or_else(|| {
            std::env::var("IQOS_SCAN_TIMEOUT")
                .ok()
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(10);

    Duration::from_secs(seconds)
}

pub fn should_use_cli(args: &[String]) -> bool {
    args.len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_model_flags() {
        assert_eq!(parse_device_model("iluma"), Some(DeviceModel::Iluma));
        assert_eq!(
            parse_device_model("ILUMA-I-PRIME"),
            Some(DeviceModel::IlumaIPrime)
        );
        assert_eq!(
            parse_device_model("iluma_i_one"),
            Some(DeviceModel::IlumaIOne)
        );
    }

    #[test]
    fn leaves_unknown_model_values_for_label_resolution() {
        assert_eq!(parse_device_model("blackcat"), None);
    }

    #[test]
    fn converts_registered_commands_to_repl_arg_shape() {
        let command = CliCommand::Vibration {
            args: vec!["heating".to_string(), "on".to_string()],
        }
        .into_one_shot();

        assert_eq!(
            command,
            OneShotCommand::Registered {
                name: "vibration",
                args: vec![
                    "vibration".to_string(),
                    "heating".to_string(),
                    "on".to_string()
                ],
            }
        );
    }

    #[test]
    fn passthrough_commands_accept_hyphen_prefixed_values() {
        let cli = Cli::try_parse_from(["iqos", "vibration", "heating", "-badflag", "--also-value"])
            .unwrap();

        assert_eq!(
            cli.command.map(CliCommand::into_one_shot),
            Some(OneShotCommand::Registered {
                name: "vibration",
                args: vec![
                    "vibration".to_string(),
                    "heating".to_string(),
                    "-badflag".to_string(),
                    "--also-value".to_string(),
                ],
            })
        );
    }

    #[test]
    fn any_argument_uses_cli_mode() {
        assert!(!should_use_cli(&["iqos".to_string()]));
        assert!(should_use_cli(&["iqos".to_string(), "--help".to_string()]));
        assert!(should_use_cli(&["iqos".to_string(), "battery".to_string()]));
    }
}

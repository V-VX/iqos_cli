use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceCapability, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::{invalid_arguments, IQOSConsole};

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "autostart",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutostartAction {
    Enable,
    Disable,
    Status,
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let action = parse_action(&args)?;
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !model.supports(DeviceCapability::AutoStart) {
        println!("Autostart is not supported on this device");
        return Ok(());
    }

    match action {
        AutostartAction::Enable => {
            iqos.set_autostart(model, true).await?;
            println!("Autostart enabled");
        }
        AutostartAction::Disable => {
            iqos.set_autostart(model, false).await?;
            println!("Autostart disabled");
        }
        AutostartAction::Status => {
            let enabled = iqos.read_autostart(model).await?;
            println!(
                "Autostart: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    Ok(())
}

fn parse_action(args: &[String]) -> Result<AutostartAction> {
    match args.get(1).map(String::as_str) {
        None => Ok(AutostartAction::Status),
        Some("status") if args.len() == 2 => Ok(AutostartAction::Status),
        Some("on") | Some("enable") if args.len() == 2 => Ok(AutostartAction::Enable),
        Some("off") | Some("disable") if args.len() == 2 => Ok(AutostartAction::Disable),
        Some(_) if args.len() > 2 => Err(invalid_arguments(
            "Usage: autostart [enable|on|disable|off|status]",
        )),
        Some(opt) => Err(invalid_arguments(format!(
            "Invalid option: {opt}. Use enable/on, disable/off, or status (default)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iqos::DeviceModel;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_enable() {
        assert_eq!(
            parse_action(&args(&["autostart", "enable"])).unwrap(),
            AutostartAction::Enable
        );
        assert_eq!(
            parse_action(&args(&["autostart", "on"])).unwrap(),
            AutostartAction::Enable
        );
    }

    #[test]
    fn parses_disable() {
        assert_eq!(
            parse_action(&args(&["autostart", "disable"])).unwrap(),
            AutostartAction::Disable
        );
        assert_eq!(
            parse_action(&args(&["autostart", "off"])).unwrap(),
            AutostartAction::Disable
        );
    }

    #[test]
    fn parses_status() {
        assert_eq!(
            parse_action(&args(&["autostart", "status"])).unwrap(),
            AutostartAction::Status
        );
        assert_eq!(
            parse_action(&args(&["autostart"])).unwrap(),
            AutostartAction::Status
        );
    }

    #[test]
    fn rejects_trailing_args() {
        assert!(parse_action(&args(&["autostart", "enable", "typo"])).is_err());
        assert!(parse_action(&args(&["autostart", "disable", "typo"])).is_err());
        assert!(parse_action(&args(&["autostart", "status", "typo"])).is_err());
    }

    #[test]
    fn rejects_invalid() {
        assert!(parse_action(&args(&["autostart", "bogus"])).is_err());
    }

    #[test]
    fn autostart_capability_matches_iluma_i_series() {
        for model in [
            DeviceModel::IlumaI,
            DeviceModel::IlumaIOne,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(model.supports(DeviceCapability::AutoStart));
        }

        for model in [
            DeviceModel::Iluma,
            DeviceModel::IlumaOne,
            DeviceModel::IlumaPrime,
        ] {
            assert!(!model.supports(DeviceCapability::AutoStart));
        }
    }
}

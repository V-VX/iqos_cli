use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceCapability, FlexPuffSetting, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::{invalid_arguments, IQOSConsole};

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "flexpuff",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexPuffAction {
    Enable,
    Disable,
    Status,
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let action = parse_action(&args)?;
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !model.supports(DeviceCapability::FlexPuff) {
        println!("FlexPuff is not supported on this device");
        return Ok(());
    }

    match action {
        FlexPuffAction::Enable => {
            iqos.set_flexpuff(model, FlexPuffSetting::new(true)).await?;
            println!("FlexPuff enabled");
        }
        FlexPuffAction::Disable => {
            iqos.set_flexpuff(model, FlexPuffSetting::new(false))
                .await?;
            println!("FlexPuff disabled");
        }
        FlexPuffAction::Status => {
            let s = iqos.read_flexpuff(model).await?;
            println!(
                "FlexPuff: {}",
                if s.is_enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }

    Ok(())
}

fn parse_action(args: &[String]) -> Result<FlexPuffAction> {
    match args.get(1).map(String::as_str) {
        None if args.len() == 1 => Ok(FlexPuffAction::Status),
        Some("enable") if args.len() == 2 => Ok(FlexPuffAction::Enable),
        Some("enable") => Err(invalid_arguments("Usage: flexpuff enable")),
        Some("disable") if args.len() == 2 => Ok(FlexPuffAction::Disable),
        Some("disable") => Err(invalid_arguments("Usage: flexpuff disable")),
        Some("status") if args.len() == 2 => Ok(FlexPuffAction::Status),
        Some("status") => Err(invalid_arguments("Usage: flexpuff status")),
        Some(opt) => Err(invalid_arguments(format!(
            "Invalid option: {opt}. Use enable/disable/status"
        ))),
        None => Err(invalid_arguments("Usage: flexpuff [enable|disable|status]")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn parses_enable() {
        assert_eq!(
            parse_action(&args(&["flexpuff", "enable"])).unwrap(),
            FlexPuffAction::Enable
        );
    }

    #[test]
    fn parses_disable() {
        assert_eq!(
            parse_action(&args(&["flexpuff", "disable"])).unwrap(),
            FlexPuffAction::Disable
        );
    }

    #[test]
    fn parses_status_subcommand() {
        assert_eq!(
            parse_action(&args(&["flexpuff", "status"])).unwrap(),
            FlexPuffAction::Status
        );
    }

    #[test]
    fn parses_default_status() {
        assert_eq!(
            parse_action(&args(&["flexpuff"])).unwrap(),
            FlexPuffAction::Status
        );
    }

    #[test]
    fn rejects_trailing_args() {
        assert!(parse_action(&args(&["flexpuff", "enable", "typo"])).is_err());
        assert!(parse_action(&args(&["flexpuff", "disable", "typo"])).is_err());
        assert!(parse_action(&args(&["flexpuff", "status", "typo"])).is_err());
    }

    #[test]
    fn rejects_invalid_subcommand() {
        assert!(parse_action(&args(&["flexpuff", "invalid"])).is_err());
    }
}

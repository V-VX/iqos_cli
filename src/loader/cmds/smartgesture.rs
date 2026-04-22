use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::compat::supports_smartgesture;
use crate::loader::parser::{invalid_arguments, IQOSConsole};

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "smartgesture",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmartGestureAction {
    Enable,
    Disable,
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let action = parse_action(&args)?;
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !supports_smartgesture(model) {
        println!("SmartGesture is only available on ILUMA holder models");
        return Ok(());
    }

    match action {
        SmartGestureAction::Enable => {
            iqos.set_smartgesture(model, true).await?;
            println!("Smart Gesture enabled");
        }
        SmartGestureAction::Disable => {
            iqos.set_smartgesture(model, false).await?;
            println!("Smart Gesture disabled");
        }
    }

    Ok(())
}

fn parse_action(args: &[String]) -> Result<SmartGestureAction> {
    match args.get(1).map(String::as_str) {
        Some("enable") if args.len() == 2 => Ok(SmartGestureAction::Enable),
        Some("enable") => Err(invalid_arguments("Usage: smartgesture enable")),
        Some("disable") if args.len() == 2 => Ok(SmartGestureAction::Disable),
        Some("disable") => Err(invalid_arguments("Usage: smartgesture disable")),
        Some(opt) => Err(invalid_arguments(format!(
            "Invalid option: {opt}. Use enable/disable"
        ))),
        None => Err(invalid_arguments("Usage: smartgesture [enable|disable]")),
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
            parse_action(&args(&["smartgesture", "enable"])).unwrap(),
            SmartGestureAction::Enable
        );
    }

    #[test]
    fn parses_disable() {
        assert_eq!(
            parse_action(&args(&["smartgesture", "disable"])).unwrap(),
            SmartGestureAction::Disable
        );
    }

    #[test]
    fn rejects_trailing_args() {
        assert!(parse_action(&args(&["smartgesture", "enable", "typo"])).is_err());
        assert!(parse_action(&args(&["smartgesture", "disable", "typo"])).is_err());
    }

    #[test]
    fn rejects_invalid_subcommand() {
        assert!(parse_action(&args(&["smartgesture", "status"])).is_err());
    }

    #[test]
    fn rejects_missing_subcommand() {
        assert!(parse_action(&args(&["smartgesture"])).is_err());
    }
}

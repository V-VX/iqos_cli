use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

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
            println!("Autostart: {}", if enabled { "enabled" } else { "disabled" });
        }
    }

    Ok(())
}

fn parse_action(args: &[String]) -> Result<AutostartAction> {
    match args.get(1).map(String::as_str) {
        None | Some("status") => Ok(AutostartAction::Status),
        Some("on") | Some("enable") => Ok(AutostartAction::Enable),
        Some("off") | Some("disable") => Ok(AutostartAction::Disable),
        Some(opt) => bail!("Invalid option: {opt}. Use enable/disable/status"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_enable() {
        assert_eq!(parse_action(&args(&["autostart", "enable"])).unwrap(), AutostartAction::Enable);
        assert_eq!(parse_action(&args(&["autostart", "on"])).unwrap(), AutostartAction::Enable);
    }

    #[test]
    fn parses_disable() {
        assert_eq!(
            parse_action(&args(&["autostart", "disable"])).unwrap(),
            AutostartAction::Disable
        );
        assert_eq!(parse_action(&args(&["autostart", "off"])).unwrap(), AutostartAction::Disable);
    }

    #[test]
    fn parses_status() {
        assert_eq!(parse_action(&args(&["autostart", "status"])).unwrap(), AutostartAction::Status);
        assert_eq!(parse_action(&args(&["autostart"])).unwrap(), AutostartAction::Status);
    }

    #[test]
    fn rejects_invalid() {
        assert!(parse_action(&args(&["autostart", "bogus"])).is_err());
    }
}

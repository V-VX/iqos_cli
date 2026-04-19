use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle, VibrationSettings};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "vibration",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();
    let str_args: Vec<&str> = args.iter().map(String::as_str).collect();

    if args.len() == 1 {
        match iqos.read_vibration_settings(model).await {
            Ok(s) => println!("{s:?}"),
            Err(e) => println!("Error: {e}"),
        }
    } else {
        let param_args = &str_args[1..];
        let settings = if model.supports_charge_start_vibration() {
            parse_with_charge(param_args)?
        } else {
            parse_base(param_args)?
        };
        iqos.update_vibration_settings(model, settings).await?;
        println!("Vibration settings updated");
    }

    Ok(())
}

fn parse_base(args: &[&str]) -> Result<VibrationSettings> {
    let (heating, starting, puff_end, terminated) = parse_flags(args);
    Ok(VibrationSettings::new(
        heating, starting, puff_end, terminated,
    ))
}

fn parse_with_charge(args: &[&str]) -> Result<VibrationSettings> {
    let (heating, starting, puff_end, terminated) = parse_flags(args);
    let charge = flag_value(args, "charge");
    Ok(VibrationSettings::with_charge_start(
        heating, starting, puff_end, terminated, charge,
    ))
}

fn parse_flags(args: &[&str]) -> (bool, bool, bool, bool) {
    (
        flag_value(args, "heating"),
        flag_value(args, "starting"),
        flag_value(args, "puffend"),
        flag_value(args, "terminated"),
    )
}

fn flag_value(args: &[&str], key: &str) -> bool {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1] == "on")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_value_true_when_key_on() {
        assert!(flag_value(&["heating", "on"], "heating"));
    }

    #[test]
    fn flag_value_false_when_key_off() {
        assert!(!flag_value(&["heating", "off"], "heating"));
    }

    #[test]
    fn flag_value_false_when_key_absent() {
        assert!(!flag_value(&["starting", "on"], "heating"));
    }

    #[test]
    fn flag_value_false_when_empty() {
        assert!(!flag_value(&[], "heating"));
    }

    #[test]
    fn flag_value_false_when_single_element() {
        assert!(!flag_value(&["heating"], "heating"));
    }

    #[test]
    fn parse_base_returns_ok() {
        assert!(parse_base(&["heating", "on", "starting", "off"]).is_ok());
    }

    #[test]
    fn parse_base_empty_args_returns_ok_with_defaults() {
        assert!(parse_base(&[]).is_ok());
    }

    #[test]
    fn parse_with_charge_returns_ok() {
        assert!(parse_with_charge(&["charge", "on", "heating", "on"]).is_ok());
    }

    #[test]
    fn parse_with_charge_empty_returns_ok_with_defaults() {
        assert!(parse_with_charge(&[]).is_ok());
    }
}

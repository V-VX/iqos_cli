use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::{FlexBatteryMode, FlexBatterySettings, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "flexbattery",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !model.is_iluma_i_family() {
        println!("FlexBattery is only available on ILUMA i devices");
        return Ok(());
    }

    if args.len() == 1 {
        match iqos.read_flexbattery(model).await {
            Ok(s) => println!(
                "FlexBattery: mode={:?}, pause={:?}",
                s.mode(),
                s.pause_mode()
            ),
            Err(e) => println!("Error: {e}"),
        }
    } else {
        let settings = parse_args(&args[1..])?;
        iqos.set_flexbattery(model, settings).await?;
        println!("FlexBattery settings updated");
    }

    Ok(())
}

fn parse_args(args: &[String]) -> Result<FlexBatterySettings> {
    match args.first().map(String::as_str) {
        Some("performance") => Ok(FlexBatterySettings::new(FlexBatteryMode::Performance, None)),
        Some("eco") => Ok(FlexBatterySettings::new(FlexBatteryMode::Eco, None)),
        Some("pause") => {
            let enabled = args.get(1).map(|s| s == "on");
            Ok(FlexBatterySettings::new(
                FlexBatteryMode::Performance,
                enabled,
            ))
        }
        Some(s) => bail!("Invalid option: {s}. Use performance/eco/pause [on|off]"),
        None => bail!("Usage: flexbattery [performance|eco|pause on|off]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn performance_returns_ok() {
        assert!(parse_args(&args(&["performance"])).is_ok());
    }

    #[test]
    fn eco_returns_ok() {
        assert!(parse_args(&args(&["eco"])).is_ok());
    }

    #[test]
    fn pause_on_sets_pause_mode_true() {
        let s = parse_args(&args(&["pause", "on"])).unwrap();
        assert_eq!(s.pause_mode(), Some(true));
    }

    #[test]
    fn pause_off_sets_pause_mode_false() {
        let s = parse_args(&args(&["pause", "off"])).unwrap();
        assert_eq!(s.pause_mode(), Some(false));
    }

    #[test]
    fn pause_no_value_sets_pause_mode_none() {
        let s = parse_args(&args(&["pause"])).unwrap();
        assert_eq!(s.pause_mode(), None);
    }

    #[test]
    fn invalid_option_returns_err() {
        assert!(parse_args(&args(&["turbo"])).is_err());
    }

    #[test]
    fn empty_args_returns_err() {
        assert!(parse_args(&args(&[])).is_err());
    }
}

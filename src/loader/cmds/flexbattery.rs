use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::{FlexBatteryMode, FlexBatterySettings, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("flexbattery", Box::new(|iqos, args| {
        Box::pin(async move { execute(iqos, args).await })
    })).await;
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
            Ok(s) => println!("FlexBattery: mode={:?}, pause={:?}", s.mode(), s.pause_mode()),
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
            Ok(FlexBatterySettings::new(FlexBatteryMode::Performance, enabled))
        }
        Some(s) => bail!("Invalid option: {s}. Use performance/eco/pause [on|off]"),
        None => bail!("Usage: flexbattery [performance|eco|pause on|off]"),
    }
}

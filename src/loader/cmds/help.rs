use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceCapability, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "help",
        Box::new(|iqos, _| Box::pin(async move { execute(iqos).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();
    println!("Available commands:");
    println!("  battery            Display battery level");
    println!("  lock | unlock      Lock or unlock the device");
    println!("  findmyiqos         Activate find-my-device vibration");
    println!("  autostart [on|off] Configure auto-start");
    println!("  diagnosis          Retrieve telemetry data");
    if model.is_iluma_family() {
        println!("\nILUMA commands:");
        println!("  brightness [high|low]                     Set brightness");
        println!("  smartgesture [enable|disable]             Configure SmartGesture");
        println!("  flexpuff [enable|disable|status]          Configure FlexPuff");
        println!("  vibration [heating|starting|terminated|puffend] [on|off] ...");
        if model.supports(DeviceCapability::FlexBattery) {
            println!("  flexbattery [performance|eco|pause on|off]");
        }
    }
    println!("\n  info               Device information");
    println!("  help               This help");
    println!("  quit | exit        Exit");
    Ok(())
}

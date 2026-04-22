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
    println!("  device             Manage saved device labels");
    println!("  findmyiqos         Activate find-my-device vibration");
    println!("  version            Display IQOS CLI version");
    if model.supports(DeviceCapability::DeviceLock) {
        println!("  lock | unlock      Lock or unlock the device");
    }
    if model.supports(DeviceCapability::AutoStart) {
        println!("  autostart [on|off|status] Configure auto-start");
    }
    println!("  diagnosis          Retrieve telemetry data");
    let has_device_commands = model.supports(DeviceCapability::Brightness)
        || model.supports(DeviceCapability::SmartGesture)
        || model.supports(DeviceCapability::FlexPuff)
        || model.supports(DeviceCapability::Vibration)
        || model.supports(DeviceCapability::FlexBattery);

    if has_device_commands {
        println!("\nDevice commands:");
    }
    if model.supports(DeviceCapability::Brightness) {
        println!("  brightness [high|low]                     Set brightness");
    }
    if model.supports(DeviceCapability::SmartGesture) {
        println!("  smartgesture [enable|disable]             Configure SmartGesture");
    }
    if model.supports(DeviceCapability::FlexPuff) {
        println!("  flexpuff [enable|disable|status]          Configure FlexPuff");
    }
    if model.supports(DeviceCapability::Vibration) {
        println!("  vibration [heating|starting|terminated|puffend] [on|off] ...");
    }
    if model.supports(DeviceCapability::FlexBattery) {
        println!("  flexbattery [performance|eco|pause on|off]");
    }
    println!("\n  info               Device metadata, firmware, and voltage snapshot");
    println!("  help               This help");
    println!("  quit | exit        Exit");
    Ok(())
}

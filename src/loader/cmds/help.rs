use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceCapability, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::compat::{
    supports_brightness, supports_flexbattery, supports_flexpuff, supports_smartgesture,
};
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
    println!("  device-status      Display firmware and voltage snapshot");
    println!("  findmyiqos         Activate find-my-device vibration");
    if model.supports(DeviceCapability::DeviceLock) {
        println!("  lock | unlock      Lock or unlock the device");
    }
    if model.supports(DeviceCapability::AutoStart) {
        println!("  autostart [on|off|status] Configure auto-start");
    }
    println!("  diagnosis          Retrieve telemetry data");
    let has_device_commands = supports_brightness(model)
        || supports_smartgesture(model)
        || supports_flexpuff(model)
        || model.supports(DeviceCapability::Vibration)
        || supports_flexbattery(model);

    if has_device_commands {
        println!("\nDevice commands:");
    }
    if supports_brightness(model) {
        println!("  brightness [high|low]                     Set brightness");
    }
    if supports_smartgesture(model) {
        println!("  smartgesture [enable|disable]             Configure SmartGesture");
    }
    if supports_flexpuff(model) {
        println!("  flexpuff [enable|disable|status]          Configure FlexPuff");
    }
    if model.supports(DeviceCapability::Vibration) {
        println!("  vibration [heating|starting|terminated|puffend] [on|off] ...");
    }
    if supports_flexbattery(model) {
        println!("  flexbattery [performance|eco|pause on|off]");
    }
    println!("\n  info               Device information");
    println!("  help               This help");
    println!("  quit | exit        Exit");
    Ok(())
}

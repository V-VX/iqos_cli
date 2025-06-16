use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::IqosIluma;
use crate::iqos::IqosIlumaI;
use crate::iqos::flexbattery::FlexBattery;
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "autostart",
        "Configure autostart feature",
        "Usage: autostart [on|off]",
        true,  // Requires ILUMA model
        false, // Does not require ILUMA-i model
    )
}

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("autostart", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    if let Some(arg) = args.get(1) {
        match arg.to_lowercase() {
            s if s == "on" || s == "enable" => {
                let result = IqosIluma::update_autostart(&*iqos, true).await;
                match result {
                    Ok(_) => println!("Autostart enabled"),
                    Err(e) => println!("Error: {}", e),
                }
            },
            s if s == "off" || s == "disable" => {
                let result = IqosIluma::update_autostart(&*iqos, false).await;
                match result {
                    Ok(_) => println!("Autostart disabled"),
                    Err(e) => println!("Error: {}", e),
                }
            },
            _ => println!("Usage: autostart [on|off]"),
        }
    } else {
        println!("Usage: autostart [on|off]");
    }
    Ok(())
}

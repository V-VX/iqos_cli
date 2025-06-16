use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::IqosIluma;
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "smartgesture",
        "Configure Smart Gesture feature",
        "Usage: smartgesture [enable|disable]",
        true,  // Requires ILUMA model
        false, // Does not require ILUMA-i model
    )
}

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("smartgesture", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    if iqos.is_iluma_or_higher() {
        match args.get(1).map(|s| s.as_str()) {
            Some("enable") => {
                let result = IqosIluma::update_smartgesture(&*iqos, true).await;
                match result {
                    Ok(_) => println!("Smart Gesture enabled"),
                    Err(e) => println!("Error: {}", e),
                }
            },
            Some("disable") => {
                let result = IqosIluma::update_smartgesture(&*iqos, false).await;
                match result {
                    Ok(_) => println!("Smart Gesture disabled"),
                    Err(e) => println!("Error: {}", e),
                }
            },
            Some(opt) => println!("Invalid option: {}. Please specify 'enable' or 'disable'", opt),
            None => println!("Usage: smartgesture [enable|disable]"),
        }
    } else {
        println!("This device is not an ILUMA model");
    }
    Ok(())
}

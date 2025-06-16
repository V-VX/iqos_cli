use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::IqosIlumaI;
use crate::iqos::flexbattery::FlexBattery;
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

/// Get information about the flexbattery command
pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "flexbattery",
        "Configure FlexBattery feature",
        "Usage: flexbattery [performance|eco] | pause [on|off]",
        true,  // Requires ILUMA model
        true,  // Requires ILUMA-i model
    )
}

/// Register the flexbattery command
pub async fn register_command(console: &IQOSConsole) {
    console.register_command("flexbattery", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

/// Execute the flexbattery command
async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if args.len() == 1 {
        // No arguments provided, show current flexbattery mode
        if let Some(iluma_i) = iqos.as_iluma_i() {
            match iluma_i.load_flexbattery().await {
                Ok(flexbattery) => println!("\n{}\n", flexbattery),
                Err(e) => println!("Error: {}", e),
            }
        } else {
            println!("This device is not an ILUMA i model");
        }
    } else if args.len() >= 2 {
        if let Some(iluma_i) = iqos.as_iluma_i() {
            let fb = FlexBattery::from_args(&str_args[1..])?;
            iluma_i.update_flexbattery(fb).await?;
            println!("Flexbattery mode updated.");
        } else {
            println!("This device is not an ILUMA i model");
        }
    } else {
        println!("Usage: flexbattery [performance|eco] | pause [on|off]");
    }
    Ok(())
}

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::device::IqosIluma;
use crate::iqos::flexpuff::Flexpuff;
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

/// Get information about the flexpuff command
pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "flexpuff",
        "Configure FlexPuff feature",
        "Usage: flexpuff [status|enable|disable]",
        true,  // Requires ILUMA model
        false, // Does not require ILUMA-i model
    )
}

/// Register the flexpuff command
pub async fn register_command(console: &IQOSConsole) {
    console.register_command("flexpuff", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

/// Execute the flexpuff command
async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    
    // Check if device is ILUMA
    if !iqos.is_iluma_or_higher() {
        println!("FlexPuff is only available on ILUMA devices.");
        return Ok(());
    }
    
    match args.get(1).map(|s| s.as_str()) {
        Some("status") => handle_status(&iqos).await,
        Some("enable") => handle_enable(&iqos).await,
        Some("disable") => handle_disable(&iqos).await,
        Some(opt) => {
            println!("Invalid option: {}. Please specify 'enable' or 'disable'", opt);
            Ok(())
        },
        None => {
            println!("Usage: flexpuff [status|enable|disable]");
            Ok(())
        }
    }
}

/// Handle the status subcommand
async fn handle_status(iqos: &IqosBle) -> Result<()> {
    // Need to handle this as an IqosIluma trait
    if let Some(iluma) = iqos.as_iluma() {
        let status = iluma.load_flexpuff().await?;
        println!("\nFlexpuff status: {}\n", status);
    } else {
        println!("This device is not an ILUMA model");
    }
    Ok(())
}

/// Handle the enable subcommand
async fn handle_enable(iqos: &IqosBle) -> Result<()> {
    let flexpuff = Flexpuff::new(true);
    if let Some(iluma) = iqos.as_iluma() {
        match iluma.update_flexpuff(flexpuff).await {
            Ok(_) => println!("Flexpuff enabled"),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("This device is not an ILUMA model");
    }
    Ok(())
}

/// Handle the disable subcommand
async fn handle_disable(iqos: &IqosBle) -> Result<()> {
    let flexpuff = Flexpuff::new(false);
    if let Some(iluma) = iqos.as_iluma() {
        match iluma.update_flexpuff(flexpuff).await {
            Ok(_) => println!("Flexpuff disabled"),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("This device is not an ILUMA model");
    }
    Ok(())
}

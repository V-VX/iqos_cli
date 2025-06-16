use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::device::Iqos;
use crate::iqos::brightness::BrightnessLevel;
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

/// Get information about the brightness command
pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "brightness",
        "Configure device brightness level",
        "Usage: brightness [high|low]",
        true,   // Requires ILUMA model
        false,  // Does not require ILUMA-i model
    )
}

/// Register the brightness command
pub async fn register_command(console: &IQOSConsole) {
    console.register_command("brightness", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

/// Execute the brightness command
async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;

    match args.get(1).map(|s| s.parse::<BrightnessLevel>()) {
        Some(Ok(level)) => {
            if iqos.is_iluma_or_higher() {
                // Explicitly call the Iqos trait method
                Iqos::update_brightness(&*iqos, level).await?;
                println!("Set brightness to {}", level);
            } else {
                println!("This device is not an ILUMA model");
            }
        },
        Some(Err(e)) => println!("{}", e),
        None => {
            // Explicitly call the Iqos trait method
            match Iqos::load_brightness(&*iqos).await {
                Ok(level) => println!("{}", level),
                Err(_) => println!("This device is not an ILUMA model")
            }
        },
    }
    Ok(())
}
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::device::{Iqos, IqosIluma};
use crate::iqos::vibration::{VibrationBehavior, VibrationSettings, IlumaVibrationBehavior};
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

/// Get information about the vibration command
pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "vibration",
        "Configure device vibration settings",
        "Usage: vibration [charge|heating|starting|terminated|puffend] [on|off] ...\nExample: vibration charge on heating on puffend on\nNote: charge option is only available for ILUMA models",
        false,  // Does not require ILUMA model
        false,  // Does not require ILUMA-i model
    )
}

/// Register the vibration command
pub async fn register_command(console: &IQOSConsole) {
    console.register_command("vibration", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

/// Execute the vibration command
async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    if str_args.len() >= 2 {
        let param_args = &str_args[1..];
        
        if iqos.is_iluma_or_higher() {
            let settings = VibrationSettings::from_args_with_charge_start(param_args)?;
            IqosIluma::update_iluma_vibration_settings(&*iqos, settings).await?;
            println!("Vibration settings updated");
        } else {
            let settings = VibrationSettings::from_args(param_args)?;
            Iqos::update_vibration_settings(&*iqos, settings).await?;
            println!("Vibration settings updated");
        }
    } else if args.len() == 1 {
        if iqos.is_iluma_or_higher() {
            match IqosIluma::load_iluma_vibration_settings(&*iqos).await {
                Ok(settings) => {
                    println!("{}", settings);
                },
                Err(e) => println!("Error: {}", e),
            }
        } else {
            match Iqos::load_vibration_settings(&*iqos).await {
                Ok(settings) => println!("{}", settings),
                Err(e) => println!("Error: {}", e),
            }
        }
    } else {
        println!("Usage: vibration [charge|heating|starting|terminated|puffend] [on|off] ...");
        println!("Example: vibration charge on heating on puffend on");
        println!("Note: charge option is only available for ILUMA models");
    }
    Ok(())
}

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;
use crate::iqos::device::{Iqos};
use crate::loader::parser::IQOSConsole;

use super::command::{CommandRegistry, CommandInfo};

pub fn command_info() -> CommandInfo {
    CommandInfo::new(
        "telemetry",
        "Retrieve telemetry data from the device",
        "Usage: telemetry",
        false,
        false,
    )
}

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("telemetry", Box::new(|iqos, args| {
        Box::pin(async move {
            execute_command(iqos, args).await
        })
    })).await;
}

pub async fn execute_command(iqos: Arc<Mutex<IqosBle>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let result = Iqos::telemetry(&*iqos).await;
    match result {
        Ok(_) => println!("Telemetry data retrieved successfully."),
        Err(e) => println!("Error retrieving telemetry data: {}", e),
    }
    Ok(())
}

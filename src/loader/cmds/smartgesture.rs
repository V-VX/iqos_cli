use std::sync::Arc;

use anyhow::Result;
use iqos::protocol::smartgesture_command;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::compat::supports_smartgesture;
use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "smartgesture",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !supports_smartgesture(model) {
        println!("SmartGesture is only available on ILUMA i series devices");
        return Ok(());
    }

    match args.get(1).map(String::as_str) {
        Some("enable") => {
            iqos.transport().send(smartgesture_command(true)).await?;
            println!("Smart Gesture enabled");
        }
        Some("disable") => {
            iqos.transport().send(smartgesture_command(false)).await?;
            println!("Smart Gesture disabled");
        }
        Some(opt) => println!("Invalid option: {opt}. Use enable/disable"),
        None => println!("Usage: smartgesture [enable|disable]"),
    }

    Ok(())
}

use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

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

    if !model.is_iluma_family() {
        println!("SmartGesture is only available on ILUMA devices");
        return Ok(());
    }

    match args.get(1).map(String::as_str) {
        Some("enable") => {
            iqos.set_smartgesture(model, true).await?;
            println!("Smart Gesture enabled");
        }
        Some("disable") => {
            iqos.set_smartgesture(model, false).await?;
            println!("Smart Gesture disabled");
        }
        Some(opt) => println!("Invalid option: {opt}. Use enable/disable"),
        None => println!("Usage: smartgesture [enable|disable]"),
    }

    Ok(())
}

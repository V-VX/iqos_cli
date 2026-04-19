use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "autostart",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    match args.get(1).map(String::as_str) {
        Some("on") | Some("enable") => {
            iqos.set_autostart(model, true).await?;
            println!("Autostart enabled");
        }
        Some("off") | Some("disable") => {
            iqos.set_autostart(model, false).await?;
            println!("Autostart disabled");
        }
        _ => println!("Usage: autostart [on|off]"),
    }

    Ok(())
}

use std::sync::Arc;

use anyhow::Result;
use iqos::{FlexPuffSetting, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("flexpuff", Box::new(|iqos, args| {
        Box::pin(async move { execute(iqos, args).await })
    })).await;
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;

    if !iqos.transport().model().is_iluma_family() {
        println!("FlexPuff is only available on ILUMA devices");
        return Ok(());
    }

    match args.get(1).map(String::as_str) {
        Some("enable") => {
            iqos.set_flexpuff(FlexPuffSetting::new(true)).await?;
            println!("FlexPuff enabled");
        }
        Some("disable") => {
            iqos.set_flexpuff(FlexPuffSetting::new(false)).await?;
            println!("FlexPuff disabled");
        }
        Some("status") | None => match iqos.read_flexpuff().await {
            Ok(s) => println!("FlexPuff: {}", if s.is_enabled() { "enabled" } else { "disabled" }),
            Err(e) => println!("Error: {e}"),
        },
        Some(opt) => println!("Invalid option: {opt}. Use enable/disable/status"),
    }

    Ok(())
}

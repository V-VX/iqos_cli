use std::sync::Arc;

use anyhow::Result;
use iqos::{BrightnessLevel, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub async fn register_command(console: &IQOSConsole) {
    console.register_command("brightness", Box::new(|iqos, args| {
        Box::pin(async move { execute(iqos, args).await })
    })).await;
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;

    if !iqos.transport().model().is_iluma_family() {
        println!("Brightness not supported on this device");
        return Ok(());
    }

    match args.get(1).map(|s| s.parse::<BrightnessLevel>()) {
        Some(Ok(level)) => {
            iqos.set_brightness(level).await?;
            println!("Brightness set to {level}");
        }
        Some(Err(e)) => println!("{e}"),
        None => match iqos.read_brightness().await {
            Ok(level) => println!("Brightness: {level}"),
            Err(e) => println!("Error: {e}"),
        },
    }

    Ok(())
}

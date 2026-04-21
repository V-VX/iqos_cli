use std::sync::Arc;

use anyhow::Result;
use iqos::{FlexPuffSetting, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::compat::supports_flexpuff;
use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "flexpuff",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;

    if !supports_flexpuff(iqos.transport().model()) {
        println!("FlexPuff is only available on ILUMA i series devices");
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
        Some("status") | None => {
            let s = iqos.read_flexpuff().await?;
            println!(
                "FlexPuff: {}",
                if s.is_enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        Some(opt) => println!("Invalid option: {opt}. Use enable/disable/status"),
    }

    Ok(())
}

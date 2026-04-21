use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "lock",
        Box::new(|iqos, _| Box::pin(async move { execute(iqos).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>) -> Result<()> {
    let iqos = iqos.lock().await;
    iqos.lock(iqos.transport().model()).await?;
    println!("Device locked");
    Ok(())
}

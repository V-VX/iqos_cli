use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "info",
        Box::new(|iqos, _| Box::pin(async move { execute(iqos).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();
    let info = iqos.transport().device_info();
    println!("\nModel:        {:?}", model);
    println!("Serial:       {}", info.serial_number.as_deref().unwrap_or("N/A"));
    println!("Software:     {}", info.software_revision.as_deref().unwrap_or("N/A"));
    println!("Manufacturer: {}", info.manufacturer_name.as_deref().unwrap_or("N/A"));
    println!();
    Ok(())
}

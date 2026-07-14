use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosTransport};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "battery",
        Box::new(|iqos, _| Box::pin(async move { execute(iqos).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosTransport>>>) -> Result<()> {
    let iqos = iqos.lock().await;
    match iqos.transport() {
        IqosTransport::Ble(transport) => {
            let level = transport.read_battery_level().await?;
            println!("Battery: {level}%");
        }
        IqosTransport::Usb(_) => {
            let voltage = iqos.read_battery_voltage().await?;
            println!("Battery voltage: {voltage:.3}V");
        }
    }
    Ok(())
}

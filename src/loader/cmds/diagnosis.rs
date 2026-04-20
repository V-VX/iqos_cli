use std::sync::Arc;

use anyhow::Result;
use iqos::{Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "diagnosis",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, _args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let data = iqos.read_diagnosis().await?;
    println!("Diagnosis:");
    if let Some(count) = data.total_smoking_count {
        println!("  Total puffs:     {count}");
    }
    if let Some(days) = data.days_used {
        println!("  Days used:       {days}");
    }
    if let Some(volts) = data.battery_voltage {
        println!("  Battery voltage: {volts:.2}V");
    }
    Ok(())
}

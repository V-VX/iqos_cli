use std::sync::Arc;

use anyhow::Result;
use iqos::{DeviceStatus, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::{invalid_arguments, IQOSConsole};

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "info",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err(invalid_arguments("Usage: info"));
    }

    let iqos = iqos.lock().await;
    let model = iqos.transport().model();
    let device_info = iqos.transport().device_info().clone();
    let status = iqos.read_device_status(model, device_info).await?;

    print_status(&status);
    Ok(())
}

fn print_status(status: &DeviceStatus) {
    let info = &status.device_info;

    println!("Device Information:");
    println!("  Model:           {:?}", status.model);
    println!(
        "  Model number:    {}",
        field_or_missing(info.model_number.as_deref())
    );
    println!(
        "  Serial number:   {}",
        field_or_missing(info.serial_number.as_deref())
    );
    println!(
        "  Manufacturer:    {}",
        field_or_missing(info.manufacturer_name.as_deref())
    );
    println!(
        "  Software rev:    {}",
        field_or_missing(info.software_revision.as_deref())
    );
    println!("  Product number:  {}", status.product_number);
    println!("  Stick firmware:  {}", status.stick_firmware);

    if let Some(product_number) = &status.holder_product_number {
        println!("  Holder product:  {product_number}");
    }
    if let Some(firmware) = &status.holder_firmware {
        println!("  Holder firmware: {firmware}");
    }

    match status.battery_voltage {
        Some(voltage) => println!("  Battery voltage: {voltage:.3}V"),
        None => println!("  Battery voltage: read failed"),
    }
}

fn field_or_missing(value: Option<&str>) -> &str {
    value.unwrap_or("N/A")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_field_falls_back_to_na() {
        assert_eq!(field_or_missing(None), "N/A");
    }

    #[test]
    fn present_field_is_preserved() {
        assert_eq!(field_or_missing(Some("value")), "value");
    }
}

use anyhow::{bail, Result};

use crate::config::{
    normalize_device_label, print_saved_devices, validate_device_label, AppConfig, ConnectedDevice,
};
use crate::loader::parser::invalid_arguments;

pub async fn execute(args: Vec<String>, connected_device: Option<&ConnectedDevice>) -> Result<()> {
    match args.get(1).map(String::as_str) {
        Some("list") if args.len() == 2 => list_devices(),
        Some("list") => Err(invalid_arguments("Usage: device list")),
        Some("save") if args.len() == 3 => save_device(&args[2], connected_device),
        Some("save") => Err(invalid_arguments("Usage: device save <label>")),
        Some("remove") if args.len() == 3 => remove_device(&args[2]),
        Some("remove") => Err(invalid_arguments("Usage: device remove <label>")),
        Some(subcommand) => Err(invalid_arguments(format!(
            "Invalid option: {subcommand}. Use list/save/remove"
        ))),
        None => Err(invalid_arguments("Usage: device [list|save|remove]")),
    }
}

fn list_devices() -> Result<()> {
    let config = AppConfig::load()?;
    print_saved_devices(&config);
    Ok(())
}

fn save_device(label: &str, connected_device: Option<&ConnectedDevice>) -> Result<()> {
    let Some(device) = connected_device else {
        bail!("No connected device metadata available");
    };

    let mut config = AppConfig::load()?;
    let label =
        validate_device_label(label).map_err(|error| invalid_arguments(error.to_string()))?;
    config.save_device(label.clone(), device)?;
    config.update_default(device);
    config.save()?;
    println!("Saved device label: {label}");
    Ok(())
}

fn remove_device(label: &str) -> Result<()> {
    let mut config = AppConfig::load()?;
    let label =
        normalize_device_label(label).map_err(|error| invalid_arguments(error.to_string()))?;
    if !config.remove_device(&label)? {
        bail!("Device label not found: {label}");
    }

    config.save()?;
    println!("Removed device label: {label}");
    Ok(())
}

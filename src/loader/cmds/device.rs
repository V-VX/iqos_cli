use anyhow::{bail, Result};

use crate::config::{print_saved_devices, AppConfig, ConnectedDevice};

pub async fn execute(args: Vec<String>, connected_device: Option<&ConnectedDevice>) -> Result<()> {
    match args.get(1).map(String::as_str) {
        Some("list") if args.len() == 2 => list_devices(),
        Some("list") => bail!("Usage: device list"),
        Some("save") if args.len() == 3 => save_device(&args[2], connected_device),
        Some("save") => bail!("Usage: device save <label>"),
        Some("remove") if args.len() == 3 => remove_device(&args[2]),
        Some("remove") => bail!("Usage: device remove <label>"),
        Some(subcommand) => bail!("Invalid option: {subcommand}. Use list/save/remove"),
        None => bail!("Usage: device [list|save|remove]"),
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
    config.update_default(device);
    config.save_device(label.to_string(), device);
    config.save()?;
    println!("Saved device label: {label}");
    Ok(())
}

fn remove_device(label: &str) -> Result<()> {
    let mut config = AppConfig::load()?;
    if !config.remove_device(label) {
        bail!("Device label not found: {label}");
    }

    config.save()?;
    println!("Removed device label: {label}");
    Ok(())
}

use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::{FlexBatteryMode, FlexBatterySettings, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "flexbattery",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !model.is_iluma_i_family() {
        println!("FlexBattery is only available on ILUMA i devices");
        return Ok(());
    }

    let settings = match args.get(1).map(String::as_str) {
        None => {
            let s = iqos.read_flexbattery(model).await?;
            println!("FlexBattery: mode={:?}, pause={:?}", s.mode(), s.pause_mode());
            return Ok(());
        }
        Some("pause") => {
            let pause = parse_on_off(args.get(2).map(String::as_str))?;
            // Read current mode so toggling pause doesn't overwrite it.
            let current = iqos.read_flexbattery(model).await?;
            FlexBatterySettings::new(current.mode(), pause)
        }
        Some("performance") => FlexBatterySettings::new(FlexBatteryMode::Performance, None),
        Some("eco") => FlexBatterySettings::new(FlexBatteryMode::Eco, None),
        Some(s) => bail!("Invalid option: {s}. Use performance/eco/pause [on|off]"),
    };

    iqos.set_flexbattery(model, settings).await?;
    println!("FlexBattery settings updated");
    Ok(())
}

fn parse_on_off(value: Option<&str>) -> Result<Option<bool>> {
    match value {
        Some("on") => Ok(Some(true)),
        Some("off") => Ok(Some(false)),
        None => Ok(None),
        Some(s) => bail!("Invalid pause value: {s}. Use on/off"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_on_parsed() {
        assert_eq!(parse_on_off(Some("on")).unwrap(), Some(true));
    }

    #[test]
    fn pause_off_parsed() {
        assert_eq!(parse_on_off(Some("off")).unwrap(), Some(false));
    }

    #[test]
    fn pause_absent_is_none() {
        assert_eq!(parse_on_off(None).unwrap(), None);
    }

    #[test]
    fn pause_invalid_returns_err() {
        assert!(parse_on_off(Some("yes")).is_err());
    }
}

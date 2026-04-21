use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::protocol::{LOAD_FLEXBATTERY_COMMAND, LOAD_PAUSEMODE_COMMAND};
use iqos::{FlexBatteryMode, FlexBatterySettings, Iqos, IqosBle};
use tokio::sync::Mutex;

use crate::loader::compat::supports_flexbattery;
use crate::loader::parser::IQOSConsole;

const PAUSEMODE_ENABLE_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x05];
const PAUSEMODE_DISABLE_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x02, 0x00, 0x00, 0x00, 0x6E];

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "flexbattery",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();

    if !supports_flexbattery(model) {
        println!("FlexBattery is only available on ILUMA i and ILUMA i PRIME devices");
        return Ok(());
    }

    let cmd = args.get(1).map(|s| s.to_ascii_lowercase());
    let settings = match cmd.as_deref() {
        None => {
            let s = read_flexbattery(&iqos).await?;
            println!(
                "FlexBattery: mode={:?}, pause={:?}",
                s.mode(),
                s.pause_mode()
            );
            return Ok(());
        }
        Some("pause") => {
            if args.len() != 3 {
                bail!("Usage: flexbattery pause [on|off]");
            }
            let value = args.get(2).map(|s| s.to_ascii_lowercase());
            let pause = parse_on_off(value.as_deref())?;
            let current = read_flexbattery(&iqos).await?;
            FlexBatterySettings::new(current.mode(), pause)
        }
        Some("performance") => {
            if args.len() != 2 {
                bail!("Usage: flexbattery performance");
            }
            FlexBatterySettings::new(FlexBatteryMode::Performance, None)
        }
        Some("eco") => {
            if args.len() != 2 {
                bail!("Usage: flexbattery eco");
            }
            FlexBatterySettings::new(FlexBatteryMode::Eco, None)
        }
        Some(s) => bail!("Invalid option: {s}. Use performance/eco/pause [on|off]"),
    };

    set_flexbattery(&iqos, settings).await?;
    println!("FlexBattery settings updated");
    Ok(())
}

async fn read_flexbattery(iqos: &Iqos<IqosBle>) -> Result<FlexBatterySettings> {
    let mode_response = iqos.transport().request(&LOAD_FLEXBATTERY_COMMAND).await?;
    let pause_response = iqos.transport().request(&LOAD_PAUSEMODE_COMMAND).await?;
    Ok(FlexBatterySettings::from_responses(
        &mode_response,
        &pause_response,
    )?)
}

async fn set_flexbattery(iqos: &Iqos<IqosBle>, settings: FlexBatterySettings) -> Result<()> {
    iqos.transport()
        .send(settings.mode().write_command())
        .await?;
    iqos.transport().send(&LOAD_FLEXBATTERY_COMMAND).await?;
    if let Some(pause_mode) = settings.pause_mode() {
        iqos.transport().send(pausemode_command(pause_mode)).await?;
        iqos.transport().send(&LOAD_PAUSEMODE_COMMAND).await?;
    }
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

fn pausemode_command(enabled: bool) -> &'static [u8] {
    if enabled {
        &PAUSEMODE_ENABLE_SET_COMMAND
    } else {
        &PAUSEMODE_DISABLE_SET_COMMAND
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

    #[test]
    fn pausemode_command_selects_correct_command() {
        assert_eq!(pausemode_command(true), &PAUSEMODE_ENABLE_SET_COMMAND);
        assert_eq!(pausemode_command(false), &PAUSEMODE_DISABLE_SET_COMMAND);
    }
}

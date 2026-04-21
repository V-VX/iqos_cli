use std::sync::Arc;

use anyhow::{bail, Result};
use iqos::{Iqos, IqosBle, VibrationSettings};
use tokio::sync::Mutex;

use crate::loader::parser::IQOSConsole;

pub fn register_command(console: &mut IQOSConsole) {
    console.register_command(
        "vibration",
        Box::new(|iqos, args| Box::pin(async move { execute(iqos, args).await })),
    );
}

async fn execute(iqos: Arc<Mutex<Iqos<IqosBle>>>, args: Vec<String>) -> Result<()> {
    let iqos = iqos.lock().await;
    let model = iqos.transport().model();
    let str_args: Vec<&str> = args.iter().map(String::as_str).collect();

    if args.len() == 1 {
        let s = iqos.read_vibration_settings(model).await?;
        println!("{s:?}");
        return Ok(());
    }

    let param_args = &str_args[1..];
    validate_flags(param_args, model.supports_charge_start_vibration())?;
    let current = iqos.read_vibration_settings(model).await?;
    let settings = apply_changes(current, param_args, model.supports_charge_start_vibration())?;
    iqos.update_vibration_settings(model, settings).await?;
    println!("Vibration settings updated");
    Ok(())
}

fn validate_flags(args: &[&str], has_charge: bool) -> Result<()> {
    const VALID: &[&str] = &["heating", "starting", "puffend", "terminated", "charge"];
    if args.len() % 2 != 0 {
        bail!("Each flag requires a value. Usage: vibration [heating|starting|puffend|terminated|charge] [on|off] ...");
    }
    for chunk in args.chunks(2) {
        if !VALID.contains(&chunk[0]) {
            bail!("Unknown flag '{}'. Valid: heating, starting, puffend, terminated, charge", chunk[0]);
        }
        if chunk[0] == "charge" && !has_charge {
            bail!("'charge' flag is not supported on this device");
        }
        if chunk[1] != "on" && chunk[1] != "off" {
            bail!("Invalid value '{}' for '{}'. Use on or off", chunk[1], chunk[0]);
        }
    }
    Ok(())
}

fn flag_update(args: &[&str], key: &str) -> Option<bool> {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1] == "on")
}

fn apply_changes(
    current: VibrationSettings,
    args: &[&str],
    has_charge: bool,
) -> Result<VibrationSettings> {
    let heating = flag_update(args, "heating").unwrap_or(current.when_heating_start());
    let starting = flag_update(args, "starting").unwrap_or(current.when_starting_to_use());
    let puff_end = flag_update(args, "puffend").unwrap_or(current.when_puff_end());
    let terminated = flag_update(args, "terminated").unwrap_or(current.when_manually_terminated());

    if has_charge {
        let charge = flag_update(args, "charge")
            .unwrap_or(current.when_charging_start().unwrap_or(false));
        Ok(VibrationSettings::with_charge_start(heating, starting, puff_end, terminated, charge))
    } else {
        Ok(VibrationSettings::new(heating, starting, puff_end, terminated))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_update_on() {
        assert_eq!(flag_update(&["heating", "on"], "heating"), Some(true));
    }

    #[test]
    fn flag_update_off() {
        assert_eq!(flag_update(&["heating", "off"], "heating"), Some(false));
    }

    #[test]
    fn flag_update_absent_is_none() {
        assert_eq!(flag_update(&["starting", "on"], "heating"), None);
    }

    #[test]
    fn flag_update_empty_is_none() {
        assert_eq!(flag_update(&[], "heating"), None);
    }

    #[test]
    fn validate_flags_rejects_odd_args() {
        assert!(validate_flags(&["heating"], false).is_err());
    }

    #[test]
    fn validate_flags_rejects_unknown_key() {
        assert!(validate_flags(&["turbo", "on"], false).is_err());
    }

    #[test]
    fn validate_flags_rejects_invalid_value() {
        assert!(validate_flags(&["heating", "yes"], false).is_err());
    }

    #[test]
    fn validate_flags_accepts_valid_pairs() {
        assert!(validate_flags(&["heating", "on", "starting", "off"], false).is_ok());
    }

    #[test]
    fn validate_flags_accepts_empty() {
        assert!(validate_flags(&[], false).is_ok());
    }

    #[test]
    fn validate_flags_rejects_charge_without_support() {
        assert!(validate_flags(&["charge", "on"], false).is_err());
    }

    #[test]
    fn validate_flags_accepts_charge_with_support() {
        assert!(validate_flags(&["charge", "on"], true).is_ok());
    }

    #[test]
    fn apply_changes_updates_only_specified_flag() {
        let current = VibrationSettings::new(true, true, false, false);
        let result = apply_changes(current, &["heating", "off"], false).unwrap();
        assert_eq!(result, VibrationSettings::new(false, true, false, false));
    }

    #[test]
    fn apply_changes_preserves_all_when_no_args() {
        let current = VibrationSettings::new(true, false, true, false);
        let result = apply_changes(current, &[], false).unwrap();
        assert_eq!(result, current);
    }
}

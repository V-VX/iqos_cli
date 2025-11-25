use futures::{stream, StreamExt};

use crate::iqos::FirmwareVersion;
use crate::iqos::error::{IQOSError, Result};
use crate::iqos::vibration::{self, VibrationSettings, LOAD_VIBRATION_SETTINGS_SIGNAL};

use super::device::IqosIluma;
use super::iqos::IqosBle;
use super::vibration::{IlumaVibration, IlumaVibrationBehavior};
use super::flexpuff::{Flexpuff, LOAD_FLEXPUFF_SIGNAL};
use btleplug::api::{Peripheral as _, WriteType};

#[derive(Default, Clone)]
pub struct IlumaSpecific {
    holder_product_number: String,
    firmware_version: FirmwareVersion,
}

impl IlumaSpecific {
    pub fn holder_product_number(&self) -> &str {
        &self.holder_product_number
    }

    pub fn with_holder_product_number(mut self, new: &str) -> Self {
        self.holder_product_number = new.to_string();
        self
    }

    pub fn firmware_version(&self) -> &FirmwareVersion {
        &self.firmware_version
    }

    pub fn with_firmware_version(mut self, new: FirmwareVersion) -> Self {
        self.firmware_version = new;
        self
    }
}

pub const FLEXPUFF_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x01, 0x00, 0x00, 0x0A];
pub const FLEXPUFF_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x00, 0x00, 0x00, 0x0A];
pub const AUTOSTART_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3f];
pub const AUTOSTART_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54];

pub const SMARTGESTURE_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3c];
pub const SMARTGESTURE_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57];

const LOAD_VIBRATE_CHARGE_START_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08];

#[derive(Debug)]
pub struct NotIlumaError;

impl std::fmt::Display for NotIlumaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This device is not an IQOS ILUMA model")
    }
}

impl std::error::Error for NotIlumaError {}

impl IqosIluma for IqosBle {
    async fn load_iluma_vibration_settings(&self) -> Result<VibrationSettings> {
        let mut when_charge_start: IlumaVibration = IlumaVibration::new(false);
        let mut vibration_settings: VibrationSettings = VibrationSettings::new(
            false,
            false,
            false,
            false,
        );
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }
        self.send_command(LOAD_VIBRATE_CHARGE_START_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;
        if let Some(notification) = stream.next().await {
            if let Ok(when_charge_start) = VibrationSettings::from_bytes_with_charge_start(notification.value.as_slice()) {
                vibration_settings.iluma_and_higher = Some(when_charge_start);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse vibration settings".to_string()));
            }
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("  Signal: {}", hex_string);
        }

        self.send_command(LOAD_VIBRATION_SETTINGS_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!(" Vib Signal: {}", hex_string);
            if let Ok(settings) = VibrationSettings::from_bytes(&notification.value) {
                vibration_settings.when_heating_start = settings.when_heating_start;
                vibration_settings.when_starting_to_use = settings.when_starting_to_use;
                vibration_settings.when_puff_end = settings.when_puff_end;
                vibration_settings.when_manually_terminated = settings.when_manually_terminated;
                return Ok(vibration_settings);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse vibration settings".to_string()));
            }
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }

    async fn update_iluma_vibration_settings(&self, updates: VibrationSettings) -> Result<()> {
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let current_settings = self.load_iluma_vibration_settings().await?;
        
        println!("Current settings: {}", current_settings);
        let mut new_settings = VibrationSettings::new(
            updates.when_heating_start.unwrap_or(current_settings.when_heating_start()),
            updates.when_starting_to_use.unwrap_or(current_settings.when_starting_to_use()),
            updates.when_puff_end.unwrap_or(current_settings.when_puff_end()),
            updates.when_manually_terminated.unwrap_or(current_settings.when_manually_terminated()),
        );
        
        new_settings.iluma_and_higher = Some(updates.iluma_and_higher.unwrap_or(current_settings.iluma_vibration()));
        
        // Generate and send only the necessary signals
        let signals = IlumaVibrationBehavior::build(&new_settings);
        for (i, signal) in signals.iter().enumerate() {
            let hex_string = signal.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("  Signal {}: {}", i, hex_string);
        }
        
        for signal in signals {
            self.send_command(signal).await?;
        }

        Ok(())
    }

    async fn update_smartgesture(&self, enable: bool) -> Result<()> {
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            SMARTGESTURE_ENABLE_SIGNAL
        } else {
            SMARTGESTURE_DISABLE_SIGNAL
        };

        self.send_command(signal.to_vec()).await?;

        Ok(())
    }

    async fn update_autostart(&self, enable: bool) -> Result<()> {
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            AUTOSTART_ENABLE_SIGNAL
        } else {
            AUTOSTART_DISABLE_SIGNAL
        };

        self.send_command(signal.to_vec()).await?;

        Ok(())
    }

    async fn load_flexpuff(&self) -> Result<Flexpuff> {
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }

        self.send_command(LOAD_FLEXPUFF_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            if let Ok(settings) = Flexpuff::from_bytes(&notification.value) {
                let hex_string = notification.value.iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(" ");
                println!("  Signal: {}", hex_string);
                return Ok(settings);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse flexpuff settings".to_string()));
            }
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }

    async fn update_flexpuff(&self, setting: Flexpuff) -> Result<()> {
        if !self.is_iluma_or_higher() {
            return Err(IQOSError::IncompatibleModelError);
        }

        self.send_command(setting.to_bytes()).await?;

        Ok(())
    }
}
use std::pin::Pin;
use futures::{Stream, StreamExt};
use super::error::{IQOSError, Result};
use super::device::Iqos;
use super::iluma::IlumaSpecific;
use super::{IqosIluma, COMMAND_CHECKSUM_XOR};
use super::brightness::{BrightnessLevel, LOAD_BRIGHTNESS_SIGNAL, BRIGHTNESS_HIGH_SIGNAL, BRIGHTNESS_LOW_SIGNAL};
use super::vibration::{VibrationBehavior, VibrationSettings, LOAD_VIBRATION_SETTINGS_SIGNAL};
use super::telemetry::{LOAD_TELEMETRY_SIGNAL, Telemetry};
use crate::iqos::firmware_version::{self, FirmwareVersion};

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

pub const CONFIRMATION_SIGNAL: [u8; 5] = [0x00, 0xc0, 0x01, 0x00, 0xF6];
// pub const START_VIBRATE_SIGNAL: [u8; 8] = [0x00, 0xc0, 0x45, 0x22, 0x01, 0x1e, 0x00, 0x65];
pub const START_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x01, 0x1e, 0x00, 0x00, 0xc3];
// pub const STOP_VIBRATE_SIGNAL: [u8; 8] = [0x00, 0xc0, 0x45, 0x22, 0x00, 0x1e, 0x00, 0x64];
pub const STOP_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x00, 0x1e, 0x00, 0x00, 0xd5];
pub const LOCK_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xc9, 0x44, 0x04, 0x02, 0xff, 0x00, 0x00, 0x5a],
    &[0x00, 0xc9, 0x00, 0x04, 0x1c],
];
// pub const LOCK_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc9, 0x00, 0x04, 0xC0];
pub const UNLOCK_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xc9, 0x44, 0x04, 0x00, 0x00, 0x00, 0x00, 0x5d],
    &[0x00, 0xc9, 0x00, 0x04, 0x1c],
];
// pub const UNLOCK_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc9, 0x00, 0x04, 0xC0];

#[derive(Debug, Clone, PartialEq)]
pub enum IQOSModel {
    One,
    Iluma,
    IlumaI
}

impl std::fmt::Display for IQOSModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IQOSModel::One => write!(f, "ONE"),
            IQOSModel::Iluma => write!(f, "ILUMA"),
            IQOSModel::IlumaI => write!(f, "ILUMA i"),
        }
    }
}

impl IQOSModel {
    pub async fn from_peripheral(peripheral: &Peripheral) -> Self {
        if let Ok(properties) = peripheral.properties().await {
            if let Some(properties) = properties {
                if let Some(name) = properties.local_name {
                    if name.contains("ONE") {
                        return IQOSModel::One;
                    }
                    else if name.contains("ILUMA i") {
                        return IQOSModel::IlumaI;
                    }
                    else if name.contains("ILUMA") {
                        return IQOSModel::Iluma;
                    }
                }
            }
        }
        IQOSModel::Iluma
    }
}

pub struct IqosBle {
    modelnumber: String,
    serialnumber: String,
    softwarerevision: String,
    manufacturername: String,
    holder_battery_status: u8,
    peripheral: Peripheral,
    battery_characteristic: Characteristic,
    scp_control_characteristic: Characteristic,
    model: IQOSModel,
    product_number: String,
    firmware_version: FirmwareVersion,
    iluma: Option<IlumaSpecific>,
}

impl IqosBle {
    pub(crate) async fn new(
        peripheral: Peripheral,
        modelnumber: String,
        serialnumber: String,
        softwarerevision: String,
        manufacturername: String,
        battery_characteristic: Characteristic,
        scp_control_characteristic: Characteristic,
        product_number: String,
        firmware_version: FirmwareVersion,
        iluma: Option<IlumaSpecific>,
    ) -> Self {
        let model = IQOSModel::from_peripheral(&peripheral).await;
        Self {
            peripheral,
            modelnumber,
            serialnumber,
            softwarerevision,
            manufacturername,
            holder_battery_status: 0,
            battery_characteristic,
            scp_control_characteristic,
            model,
            product_number,
            firmware_version,
            iluma,
        }
    }

    // pub async fn notifications(&self) -> btleplug::Result<impl futures::Stream<Item = btleplug::api::ValueNotification> + Send + '_> {
    //     self.peripheral.notifications().await
    // }

    pub async fn notifications(&self) -> btleplug::Result<Pin<Box<dyn Stream<Item = btleplug::api::ValueNotification> + Send + '_>>> {
        Ok(Box::pin(self.peripheral.notifications().await?))
    }
    
    pub async fn send_command(&self, command: Vec<u8>) -> Result<()> {
        let peripheral = &self.peripheral;
        
        peripheral.write(
            &self.scp_control_characteristic,
            &command,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;
        
        Ok(())
    }
    
    pub async fn send_command_slice<const N: usize>(&self, commands: [&[u8]; N]) -> Result<()> {
        for com in commands {
            self.send_command(com.to_vec()).await?;
        }

        Ok(())
    }

    pub async fn send_confirm(&self) -> Result<()> {
        self.send_command(CONFIRMATION_SIGNAL.to_vec()).await?;
        Ok(())
    }

    pub fn as_iluma(&self) -> Option<&IqosBle> {
        match self.model {
            IQOSModel::Iluma => Some(self),
            _ => None,
        }
    }

    pub fn is_iluma(&self) -> bool {
        matches!(self.model, IQOSModel::Iluma)
    }

    pub fn is_iluma_or_higher(&self) -> bool {
        matches!(self.model, IQOSModel::Iluma | IQOSModel::IlumaI)
    }

    pub fn is_iluma_i(&self) -> bool {
        matches!(self.model, IQOSModel::IlumaI)
    }
    
    pub fn as_iluma_i(&self) -> Option<&IqosBle> {
        match self.model {
            IQOSModel::IlumaI => Some(self),
            _ => None,
        }
    }

    pub fn model(&self) -> &IQOSModel {
        &self.model
    }

    pub(crate) fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    pub(crate) fn scp_control_characteristic(&self) -> &Characteristic {
        &self.scp_control_characteristic
    }
}

impl Iqos for IqosBle {
    async fn disconnect(&mut self) -> Result<()> {
        self.peripheral.disconnect().await.map_err(IQOSError::BleError)
    }
    
    async fn reload_battery(&mut self) -> Result<()> {
        let peripheral = &self.peripheral;

        if let Ok(data) = peripheral.read(&self.battery_characteristic)
            .await
            .map_err(IQOSError::BleError) {
                let battery_status = u8::from_str_radix(&format!("{:02X}", data[2]), 16);
                self.holder_battery_status = battery_status.unwrap_or(0);
            }
        Ok(())
    }
    
    fn battery_status(&self) -> u8 {
        self.holder_battery_status
    }
    
    async fn vibrate(&self) -> Result<()> {
        self.send_command(START_VIBRATE_SIGNAL.to_vec()).await?;
        Ok(())
    }
    
    async fn stop_vibrate(&self) -> Result<()> {
        self.send_command(STOP_VIBRATE_SIGNAL.to_vec()).await?;
        Ok(())
    }
    
    async fn lock_device(&self) -> Result<()> {
        self.send_command_slice(LOCK_SIGNALS).await?;
        self.send_confirm().await?;
        Ok(())
    }
    
    async fn unlock_device(&self) -> Result<()> {
        self.send_command_slice(UNLOCK_SIGNALS).await?;
        self.send_confirm().await?;
        Ok(())
    }
    
    async fn load_brightness(&self) -> Result<BrightnessLevel> {
        self.send_command(LOAD_BRIGHTNESS_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            
            if let Ok(settings) = BrightnessLevel::from_bytes(&notification.value) {
                return Ok(settings);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse brightness settings".to_string()));
            }
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }

    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()> {
        match level {
            BrightnessLevel::High => self.send_command_slice(BRIGHTNESS_HIGH_SIGNAL).await,
            BrightnessLevel::Low => self.send_command_slice(BRIGHTNESS_LOW_SIGNAL).await,
        }
    }

    async fn load_vibration_settings(&self) -> Result<VibrationSettings> {
        self.send_command(LOAD_VIBRATION_SETTINGS_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            
            // if let Some(iluma) = .as_iluma()
            if let Ok(settings) = VibrationSettings::from_bytes(&notification.value) {
                return Ok(settings);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse vibration settings".to_string()));
            }
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }

    async fn update_vibration_settings(&self, settings: VibrationSettings) -> Result<()> {
        let signals = settings.build();
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

    async fn telemetry(&self) -> Result<()> {
        self.send_command(LOAD_TELEMETRY_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("Telemetry Data: {}", hex_string);
            Telemetry::from_bytes(&notification.value);
            return Ok(());
        } else {
            Err(IQOSError::ConfigurationError("No telemetry data received".to_string()))
        }

    }
}

impl std::fmt::Display for IqosBle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_iluma_or_higher() {
            return write!(
                f,
                "Model: {}\nModel Number: {}\nSerial Number: {}\nManufacturer Name: {}\nFirmware version: {}\n\nStick:\n\tProduct Number: {}\n\tSoftware Revision: {}\nHolder:\n\tHolder Product Number: {}",
                self.model,
                self.modelnumber,
                self.serialnumber,
                self.manufacturername,
                self.firmware_version,
                self.product_number,
                self.softwarerevision,
                self.iluma.as_ref().unwrap().holder_product_number(),
            )
        }
        write!(
            f,
            "Model: {}\nModel Number: {}\nSerial Number: {}\nSoftware Revision: {}\nManufacturer Name: {}\nFirmware version: {}\nProduct Number: {}",
            self.model,
            self.modelnumber,
            self.serialnumber,
            self.softwarerevision,
            self.manufacturername,
            self.firmware_version,
            self.product_number,
        )
    }
}
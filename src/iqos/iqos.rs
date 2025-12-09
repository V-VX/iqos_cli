use std::pin::Pin;
use futures::{Stream, StreamExt};
use super::error::{IQOSError, Result};
use super::device::Iqos;
use super::iluma::IlumaSpecific;
use super::{IqosIluma, COMMAND_CHECKSUM_XOR};
use super::brightness::{BrightnessLevel, LOAD_BRIGHTNESS_SIGNAL, BRIGHTNESS_HIGH_SIGNAL, BRIGHTNESS_LOW_SIGNAL};
use super::vibration::{VibrationBehavior, VibrationSettings, LOAD_VIBRATION_SETTINGS_SIGNAL};
use super::diagnosis::{
    ALL_DIAGNOSIS_SIGNALS,
    Diagnosis
};
use crate::iqos::firmware_version::{FirmwareVersion};

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
    IlumaOne,
    Iluma,
    IlumaPrime,
    IlumaIOne,
    IlumaI,
    IlumaIPrime,
}

impl std::fmt::Display for IQOSModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IQOSModel::IlumaOne => write!(f, "Iluma ONE"),
            IQOSModel::Iluma => write!(f, "ILUMA"),
            IQOSModel::IlumaPrime => write!(f, "ILUMA PRIME"),
            IQOSModel::IlumaIOne => write!(f, "Iluma i ONE"),
            IQOSModel::IlumaI => write!(f, "ILUMA i"),
            IQOSModel::IlumaIPrime => write!(f, "ILUMA i PRIME"),
        }
    }
}

impl IQOSModel {
    pub async fn from_peripheral(peripheral: &Peripheral) -> Self {
        if let Ok(properties) = peripheral.properties().await {
            if let Some(properties) = properties {
                if let Some(name) = properties.local_name {
                    if name.contains("ONE") {
                        return IQOSModel::IlumaOne;
                    }
                    else if name.contains("ILUMA") {
                        return IQOSModel::Iluma;
                    }
                    else if name.contains("ILUMA Prime") {
                        return IQOSModel::IlumaPrime;
                    }
                    else if name.contains("i ONE") {
                        return IQOSModel::IlumaIOne;
                    }
                    else if name.contains("ILUMA i") {
                        return IQOSModel::IlumaI;
                    }
                    else if name.contains("ILUMA i Prime") {
                        return IQOSModel::IlumaIPrime;
                    }

                }
            }
        }
        IQOSModel::Iluma
    }

    pub fn is_iluma_one(&self) -> bool {
        matches!(self, IQOSModel::IlumaOne | IQOSModel::IlumaIOne)
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
        model: IQOSModel,
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
        Self {
            peripheral,
            model,
            modelnumber,
            serialnumber,
            softwarerevision,
            manufacturername,
            holder_battery_status: 0,
            battery_characteristic,
            scp_control_characteristic,
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

    /// Sends a command and waits for a single notification response.
    /// 
    /// This is the fundamental request-response pattern for BLE communication.
    /// Returns the raw bytes from the notification.
    async fn request(&self, command: &[u8]) -> Result<Vec<u8>> {
        #[cfg(debug_assertions)]
        {
            let cmd_hex: String = command.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
            eprintln!("[DEBUG] TX: {cmd_hex}");
        }

        self.send_command(command.to_vec()).await?;
        let mut stream = self.notifications().await?;
        
        let response = stream
            .next()
            .await
            .map(|n| n.value)
            .ok_or_else(|| IQOSError::ConfigurationError("No response received".to_string()))?;

        #[cfg(debug_assertions)]
        {
            let rx_hex: String = response.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
            eprintln!("[DEBUG] RX: {rx_hex}");
        }

        Ok(response)
    }

    /// Sends a command and parses the response using the provided parser function.
    /// 
    /// # Example
    /// ```ignore
    /// let brightness = self.request_parse(&LOAD_BRIGHTNESS_SIGNAL, BrightnessLevel::from_bytes).await?;
    /// ```
    async fn request_parse<T, F>(&self, command: &[u8], parser: F) -> Result<T>
    where
        F: FnOnce(&[u8]) -> Result<T>,
    {
        let bytes = self.request(command).await?;
        parser(&bytes)
    }

    /// Sends multiple commands sequentially, collecting all responses.
    /// 
    /// Useful for operations that require multiple request-response cycles.
    async fn request_multi(&self, commands: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        let mut responses = Vec::with_capacity(commands.len());
        for command in commands {
            responses.push(self.request(command).await?);
        }
        Ok(responses)
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
        self.request_parse(&LOAD_BRIGHTNESS_SIGNAL, BrightnessLevel::from_bytes).await
    }

    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()> {
        match level {
            BrightnessLevel::High => self.send_command_slice(BRIGHTNESS_HIGH_SIGNAL).await,
            BrightnessLevel::Low => self.send_command_slice(BRIGHTNESS_LOW_SIGNAL).await,
        }
    }

    async fn load_vibration_settings(&self) -> Result<VibrationSettings> {
        self.request_parse(&LOAD_VIBRATION_SETTINGS_SIGNAL, VibrationSettings::from_bytes).await
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

    async fn diagnosis(&self) -> Result<Diagnosis> {
        let responses = self.request_multi(ALL_DIAGNOSIS_SIGNALS).await?;
        
        responses
            .iter()
            .try_fold(Diagnosis::builder(), |builder, bytes| builder.parse(bytes))
            .map(|builder| builder.build())
    }
}

impl std::fmt::Display for IqosBle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_iluma_or_higher() {
            return write!(
                f,
                "Model: {}\nModel Number: {}\nSerial Number: {}\nManufacturer Name: {}\nFirmware version: {}\n\nStick:\n\tProduct Number: {}\n\tSoftware Revision: {}\nHolder:\n\tHolder Product Number: {}\n\tHolder Firmware version: {}",
                self.model,
                self.modelnumber,
                self.serialnumber,
                self.manufacturername,
                self.firmware_version,
                self.product_number,
                self.softwarerevision,
                self.iluma.as_ref().unwrap().holder_product_number(),
                self.iluma.as_ref().unwrap().firmware_version(),
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
use std::collections::BTreeSet;

use crate::iqos::FirmwareVersion;

use super::iluma::IlumaSpecific;
use super::iqos::{IQOSModel, IqosBle};
use super::error::{IQOSError, Result};
use super::device::{Iqos, IqosIluma};
use super::{
    BATTERY_CHARACTERISTIC_UUID, CORE_SERVICE_UUID, DEVICE_INFO_SERVICE_UUID, MANUFACTURER_NAME_CHAR_UUID, MODEL_NUMBER_CHAR_UUID, SERIAL_NUMBER_CHAR_UUID, SOFTWARE_REVISION_CHAR_UUID, SCP_CONTROL_CHARACTERISTIC_UUID, PRODUCT_NUM_SIGNAL, HOLDER_PRODUCT_NUM_SIGNAL
};
use super::firmware_version::LOAD_FIRMWARE_VERSION_SIGNAL;
use btleplug::platform::Peripheral;
use btleplug::api::{Characteristic, Peripheral as _, Service};
use uuid::Uuid;
use futures::StreamExt;

pub struct IQOSBuilder {
    peripheral: Option<Peripheral>,
    modelnumber: Option<String>,
    serialnumber: Option<String>,
    softwarerevision: Option<String>,
    manufacturername: Option<String>,
    battery_characteristic: Option<Characteristic>,
    scp_control_characteristic: Option<Characteristic>,
    product_number: Option<String>,
    firmware_version: Option<FirmwareVersion>,
    iluma: Option<IlumaSpecific>,
}

impl IQOSBuilder {
    pub fn new(peripheral: Peripheral) -> Self {
        Self {
            peripheral: Some(peripheral),
            modelnumber: None,
            serialnumber: None,
            softwarerevision: None,
            manufacturername: None,
            battery_characteristic: None,
            scp_control_characteristic: None,
            product_number: None,
            firmware_version: None,
            iluma: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        
        self.load_device_info().await?;
        self.load_characteristics().await?;

        self.subscribe(self.scp_control_characteristic.as_ref()
            .ok_or(IQOSError::ConfigurationError("SCP Control characteristic is required".to_string()))?).await?;

        self.load_product_num().await?;
        self.load_holder_product_num().await?;
        self.load_firmware_version().await?;
        
        Ok(())
    }

    async fn write(&self, byte: Vec<u8>) -> Result<()> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.write(
            self.scp_control_characteristic.as_ref().ok_or(IQOSError::ConfigurationError("SCP Control characteristic is required".to_string()))?,
            &byte,
            btleplug::api::WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;
        
        Ok(())
    }

    async fn subscribe(&self, characteristic: &Characteristic) -> Result<()> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.subscribe(characteristic).await
            .map_err(IQOSError::BleError)?;
        
        Ok(())
    }

    pub async fn discover_services(&mut self) -> Result<BTreeSet<Service>> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.discover_services().await
            .map_err(IQOSError::BleError)?;
        
        Ok(peripheral.services().into_iter().collect())
    }

    pub async fn connect(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.connect().await
            .map_err(IQOSError::BleError)?;
        Ok(())
    }

    pub async fn is_connected(&self) -> Result<bool> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.is_connected().await
            .map_err(IQOSError::BleError)
    }

    async fn load_product_num(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;

        self.write(PRODUCT_NUM_SIGNAL.to_vec()).await?;

        let mut stream = peripheral.notifications().await?;
        
        if let Some(notification) = stream.next().await {
            
            let prefix: [u8; 4] = [0x00, 0xc0, 0x88, 0x03];
            
            if notification.value.len() >= 4 && notification.value[0..4] == prefix {
                let product_num = &notification.value[4..notification.value.len() - 1];
                
                let ascii_string = product_num.iter()
                    .map(|&b| if b.is_ascii() && !b.is_ascii_control() { b as char } else { '.' })
                    .collect::<String>();
                self.product_number = Some(ascii_string);
            }
        }
        
        Ok(())
    }

    async fn load_holder_product_num(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;

        self.write(HOLDER_PRODUCT_NUM_SIGNAL.to_vec()).await?;

        let mut stream = peripheral.notifications().await?;

        if let Some(notification) = stream.next().await {
            let prefix: [u8; 4] = [0x00, 0x08, 0x88, 0x03];
            
            if notification.value.len() >= 4 && notification.value[0..4] == prefix {
                let product_num = &notification.value[4..];
                
                let ascii_string = product_num.iter()
                    .map(|&b| if b.is_ascii() && !b.is_ascii_control() { b as char } else { '.' })
                    .collect::<String>();
                println!("Holder Product Number: {}", ascii_string);
                self.iluma = Some(IlumaSpecific::new(ascii_string, "".to_string()));
            }
        }
        
        Ok(())
    }

    async fn load_firmware_version(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;

        self.write(LOAD_FIRMWARE_VERSION_SIGNAL.to_vec()).await?;
        let mut stream = peripheral.notifications().await?;

        if let Some(notification) = stream.next().await {
            if let Ok(firmware_version) = FirmwareVersion::from_bytes(&notification.value) {
                self.firmware_version = Some(firmware_version);
            }
        }
        
        Ok(())
    }

    async fn load_device_info(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        if let Some(service) = peripheral.services().iter().find(|s| s.uuid == DEVICE_INFO_SERVICE_UUID) {
            for characteristic in &service.characteristics {
                match characteristic.uuid.to_string().split('-').next().unwrap() {
                    uuid if uuid == MODEL_NUMBER_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.modelnumber = Some(value);
                            }
                        }
                    },
                    uuid if uuid == SERIAL_NUMBER_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.serialnumber = Some(value);
                            }
                        }
                    },
                    uuid if uuid == SOFTWARE_REVISION_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.softwarerevision = Some(value);
                            }
                        }
                    },
                    uuid if uuid == MANUFACTURER_NAME_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.manufacturername = Some(value);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    async fn load_characteristics(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        if let Some(service) = peripheral.services().iter().find(|s| s.uuid == CORE_SERVICE_UUID) {
            for characteristic in &service.characteristics {
                let uuid = characteristic.uuid;
                if uuid == BATTERY_CHARACTERISTIC_UUID {
                    self.battery_characteristic = Some(characteristic.clone());
                } else if uuid == SCP_CONTROL_CHARACTERISTIC_UUID {
                    self.scp_control_characteristic = Some(characteristic.clone());
                }
            }
        }
        
        Ok(())
    }

    pub async fn build(self) -> Result<IqosBle> {
        let peripheral = self.peripheral
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;

        let iqos = IqosBle::new(
            peripheral,
            self.modelnumber.unwrap_or_else(|| "Unknown".to_string()),
            self.serialnumber.ok_or(IQOSError::ConfigurationError("Serial number is required".to_string()))?,
            self.softwarerevision.unwrap_or_else(|| "Unknown".to_string()),
            self.manufacturername.unwrap_or_else(|| "Unknown".to_string()),
            self.battery_characteristic.ok_or(IQOSError::ConfigurationError("Battery characteristic is required".to_string()))?,
            self.scp_control_characteristic.ok_or(IQOSError::ConfigurationError("SCP Control characteristic is required".to_string()))?,
            self.product_number.unwrap_or_else(|| "Unknown".to_string()),
            self.firmware_version.ok_or(IQOSError::ConfigurationError("Firmware version is required".to_string()))?,
            self.iluma,
        ).await;

        Ok(iqos)
    }
}

// impl Default for IQOSBuilder {
//     fn default() -> Self {
//         Self::new(Peripheral::default())
//     }
// }
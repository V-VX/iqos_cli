use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context as _, Result};
use iqos::DeviceModel;
use serde::{Deserialize, Serialize};

use crate::model_selector::is_reserved_model_label;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<DefaultDevice>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub devices: BTreeMap<String, SavedDevice>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DefaultDevice {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SavedDevice {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedDevice {
    pub address: String,
    pub local_name: Option<String>,
    pub model: DeviceModel,
    pub serial_number: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_file();
        Self::load_from(path)
    }

    pub fn load_from(path: PathBuf) -> Result<Self> {
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents)
                .with_context(|| format!("failed to parse {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_file();
        self.save_to(path)
    }

    pub fn save_to(&self, path: PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents).with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn update_default(&mut self, device: &ConnectedDevice) {
        self.default = Some(DefaultDevice {
            address: device.address.clone(),
        });
    }

    pub fn save_device(&mut self, label: String, device: &ConnectedDevice) -> Result<()> {
        validate_device_label(&label)?;

        self.devices.insert(
            label,
            SavedDevice {
                address: device.address.clone(),
                local_name: device.local_name.clone(),
                model: Some(format!("{:?}", device.model)),
                serial_number: device.serial_number.clone(),
            },
        );

        Ok(())
    }

    pub fn update_saved_device_metadata(&mut self, label: &str, device: &ConnectedDevice) {
        if let Some(saved) = self.devices.get_mut(label) {
            saved.local_name = device
                .local_name
                .clone()
                .or_else(|| saved.local_name.clone());
            saved.model = Some(format!("{:?}", device.model));
            saved.serial_number = device
                .serial_number
                .clone()
                .or_else(|| saved.serial_number.clone());
        }
    }

    pub fn remove_device(&mut self, label: &str) -> bool {
        self.devices.remove(label).is_some()
    }
}

pub fn validate_device_label(label: &str) -> Result<()> {
    if label.trim().is_empty() {
        bail!("Invalid label: label must not be empty");
    }

    if is_reserved_model_label(label) {
        bail!("Invalid label: {label} is reserved for model selection");
    }

    Ok(())
}

pub fn config_file() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return path.join("iqos_cli").join("config.toml");
        }
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("iqos_cli").join("config.toml");
    }

    std::env::temp_dir().join("iqos_cli").join("config.toml")
}

pub fn print_saved_devices(config: &AppConfig) {
    if config.devices.is_empty() {
        println!("No saved devices");
        return;
    }

    for (label, device) in &config.devices {
        println!("{label}");
        println!("  address: {}", device.address);
        if let Some(local_name) = &device.local_name {
            println!("  local_name: {local_name}");
        }
        if let Some(model) = &device.model {
            println!("  model: {model}");
        }
        if let Some(serial_number) = &device.serial_number {
            println!("  serial_number: {serial_number}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_default_address_only() {
        let mut config = AppConfig::default();
        let device = ConnectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            local_name: Some("IQOS ILUMA i".to_string()),
            model: DeviceModel::IlumaI,
            serial_number: Some("SN123".to_string()),
        };

        config.update_default(&device);

        assert_eq!(
            config.default,
            Some(DefaultDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string()
            })
        );
    }

    #[test]
    fn saves_labelled_device_metadata() {
        let mut config = AppConfig::default();
        let device = ConnectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            local_name: Some("IQOS ILUMA i".to_string()),
            model: DeviceModel::IlumaI,
            serial_number: Some("SN123".to_string()),
        };

        config.save_device("blackcat".to_string(), &device).unwrap();

        assert_eq!(
            config.devices.get("blackcat"),
            Some(&SavedDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                local_name: Some("IQOS ILUMA i".to_string()),
                model: Some("IlumaI".to_string()),
                serial_number: Some("SN123".to_string()),
            })
        );
    }

    #[test]
    fn rejects_label_that_matches_model_selector() {
        let mut config = AppConfig::default();
        let device = ConnectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            local_name: Some("IQOS ILUMA i".to_string()),
            model: DeviceModel::IlumaI,
            serial_number: Some("SN123".to_string()),
        };

        let error = config
            .save_device("iluma-i".to_string(), &device)
            .unwrap_err();

        assert!(error.to_string().contains("reserved for model selection"));
        assert!(config.devices.is_empty());
    }
}

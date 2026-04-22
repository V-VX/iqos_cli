use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
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
        let tmp_path = path.with_extension("toml.tmp");

        let result = (|| -> Result<()> {
            let mut options = OpenOptions::new();
            options.write(true).create(true).truncate(true);

            #[cfg(unix)]
            {
                options.mode(0o600);
            }

            let mut file = options
                .open(&tmp_path)
                .with_context(|| format!("failed to write {}", tmp_path.display()))?;

            #[cfg(unix)]
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .with_context(|| format!("failed to set permissions on {}", tmp_path.display()))?;

            file.write_all(contents.as_bytes())
                .with_context(|| format!("failed to write {}", tmp_path.display()))?;
            file.sync_all()
                .with_context(|| format!("failed to sync {}", tmp_path.display()))?;
            drop(file);

            fs::rename(&tmp_path, &path)
                .with_context(|| format!("failed to replace {}", path.display()))
        })();

        if result.is_err() {
            let _ = fs::remove_file(&tmp_path);
        }

        result
    }

    pub fn update_default(&mut self, device: &ConnectedDevice) {
        self.default = Some(DefaultDevice {
            address: device.address.clone(),
        });
    }

    pub fn save_device(&mut self, label: String, device: &ConnectedDevice) -> Result<()> {
        let label = validate_device_label(&label)?;

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
        let label = label.trim();
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

    pub fn remove_device(&mut self, label: &str) -> Result<bool> {
        let label = normalize_device_label(label)?;
        Ok(self.devices.remove(&label).is_some())
    }
}

pub fn validate_device_label(label: &str) -> Result<String> {
    let trimmed = normalize_device_label(label)?;

    if is_reserved_model_label(&trimmed) {
        bail!("Invalid label: {trimmed} is reserved for model selection");
    }

    Ok(trimmed)
}

pub fn normalize_device_label(label: &str) -> Result<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        bail!("Invalid label: label must not be empty");
    }

    Ok(trimmed.to_string())
}

pub fn config_file() -> PathBuf {
    let xdg_config_home = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    config_file_from(xdg_config_home, dirs::home_dir())
}

fn config_file_from(xdg_config_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    if let Some(path) = xdg_config_home {
        if path.is_absolute() {
            return path.join("iqos_cli").join("config.toml");
        }
    }

    if let Some(home) = home {
        let xdg_path = home.join(".config").join("iqos_cli").join("config.toml");
        let legacy_path = home.join(".iqos_cli").join("config.toml");
        if legacy_path.exists() && !xdg_path.exists() {
            return legacy_path;
        }

        return xdg_path;
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
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

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
    fn trims_label_before_saving() {
        let mut config = AppConfig::default();
        let device = ConnectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            local_name: Some("IQOS ILUMA i".to_string()),
            model: DeviceModel::IlumaI,
            serial_number: Some("SN123".to_string()),
        };

        config
            .save_device("  blackcat  ".to_string(), &device)
            .unwrap();

        assert!(config.devices.contains_key("blackcat"));
        assert!(!config.devices.contains_key("  blackcat  "));
    }

    #[test]
    fn removes_device_with_trimmed_label() {
        let mut config = AppConfig::default();
        config.devices.insert(
            "blackcat".to_string(),
            SavedDevice {
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                local_name: None,
                model: None,
                serial_number: None,
            },
        );

        assert!(config.remove_device("  blackcat  ").unwrap());
        assert!(config.devices.is_empty());
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

    #[test]
    fn uses_existing_legacy_config_as_home_fallback() {
        let temp = unique_temp_dir("legacy-config");
        let legacy = temp.join(".iqos_cli").join("config.toml");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, "").unwrap();

        assert_eq!(config_file_from(None, Some(temp.clone())), legacy);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn prefers_xdg_config_when_present() {
        let temp = unique_temp_dir("xdg-config");
        let home = temp.join("home");
        let xdg = temp.join("xdg");

        assert_eq!(
            config_file_from(Some(xdg.clone()), Some(home)),
            xdg.join("iqos_cli").join("config.toml")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn save_to_restricts_config_permissions() {
        let temp = unique_temp_dir("config-permissions");
        let path = temp.join("config.toml");
        let mut config = AppConfig::default();
        let device = ConnectedDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            local_name: Some("IQOS ILUMA i".to_string()),
            model: DeviceModel::IlumaI,
            serial_number: Some("SN123".to_string()),
        };
        config.update_default(&device);

        config.save_to(path.clone()).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        assert!(!path.with_extension("toml.tmp").exists());

        fs::remove_dir_all(temp).unwrap();
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "iqos_cli_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}

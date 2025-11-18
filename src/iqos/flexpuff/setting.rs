use std::fmt;

use crate::iqos::error::{IQOSError, Result};

pub const LOAD_FLEXPUFF_SIGNAL: [u8; 9] = [0x00, 0xD2, 0x05, 0x22, 0x03, 0x00, 0x00, 0x00, 0x17];
const FLEXPUFF_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xD2, 0x45, 0x22, 0x03, 0x01, 0x00, 0x00, 0x0A];
const FLEXPUFF_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xD2, 0x45, 0x22, 0x03, 0x00, 0x00, 0x00, 0x0A];

#[derive(Default, Debug, Clone, Copy)]
pub struct Flexpuff {
    enabled: bool,
}

impl Flexpuff {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Invalid Flexpuff data".to_string()));
        }

        if bytes[0] != 0x00 || bytes[1] != 0x90 || bytes[2] != 0x85 || bytes[3] != 0x22 || bytes[4] != 0x03{
            return Err(IQOSError::ConfigurationError("Invalid Flexpuff header".to_string()));
        }

        let flag = bytes[5];
        match flag {
            0x01 => Ok(Self { enabled: true }),
            0x00 => Ok(Self { enabled: false }),
            _ => Err(IQOSError::ConfigurationError("Unknown Flexpuff state".to_string())),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        if self.enabled {
            FLEXPUFF_ENABLE_SIGNAL.to_vec()
        } else {
            FLEXPUFF_DISABLE_SIGNAL.to_vec()
        }
    }

    pub fn from_args(args: &[&str]) -> Result<Self> {
        if args.is_empty() {
            return Err(IQOSError::ConfigurationError("No arguments provided".to_string()));
        }

        match args[0].to_lowercase().as_str() {
            "enable" => Ok(Self::new(true)),
            "disable" => Ok(Self::new(false)),
            _ => Err(IQOSError::ConfigurationError("Invalid argument for Flexpuff".to_string())),
        }
    }

}

impl fmt::Display for Flexpuff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Flexpuff is {}", if self.enabled { "enabled" } else { "disabled" })
    }
}
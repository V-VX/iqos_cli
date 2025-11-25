use std::fmt;

use crate::iqos::error::{IQOSError, Result};

pub const LOAD_STICK_FIRMWARE_VERSION_SIGNAL: [u8; 7] = [0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const LOAD_HOLDER_FIRMWARE_VERSION_SIGNAL: [u8; 7] = [0x00, 0xC9, 0x00, 0x00, 0x00, 0x00, 0x00];

pub enum FirmwareKind {
    Vape = 0xC0,
    Holder = 0x08,
}

#[derive(Default, Debug, Clone)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub year: u8,
}

impl FirmwareVersion {
    pub fn from_bytes(bytes: &[u8], kind: FirmwareKind) -> Result<Self> {
        let mut firmware_version = FirmwareVersion::default();
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Data too short for firmware version".to_string()));
        }

        if bytes[0] != 0x00 || bytes[1] != kind as u8 || bytes[2] != 0x88 || bytes[3] != 0x00 {
            return Err(IQOSError::ConfigurationError("Firmware version header mismatch".to_string()));
        }

        let major = bytes[6];
        let minor = bytes[7];
        let patch = bytes[8];
        let year  = bytes[9];

        firmware_version = FirmwareVersion { major, minor, patch, year };
        Ok(firmware_version)
    }
}

impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}.{}", self.major, self.minor, self.patch, self.year)
    }
}
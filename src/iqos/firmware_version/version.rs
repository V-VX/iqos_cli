use std::fmt;

use crate::iqos::error::{IQOSError, Result};

pub const LOAD_FIRMWARE_VERSION_SIGNAL: [u8; 21] = [0x00, 0xC0, 0x01, 0x20, 0xF5, 0x55, 0x00, 0x0C, 0x00, 0x08, 0x00, 0x04, 0x00, 0x12, 0x17, 0x00, 0x00, 0xC0, 0x01, 0x20, 0xF5];

pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub year: u8,
}

impl FirmwareVersion {
    pub fn new() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            year: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut firmware_version = FirmwareVersion::new();
        if bytes.len() < 4 {
            return Err(IQOSError::ConfigurationError("Data too short for firmware version".to_string()));
        }

        if bytes[0] != 0x00 && bytes[1] != 0xC0 && bytes[2] != 0x85 && bytes[3] != 0x20 {
            return Err(IQOSError::ConfigurationError("Firmware version not available".to_string()));
        }

        let major = bytes[5];
        let minor = bytes[6];
        let patch = bytes[4];
        let year  = bytes[7];

        firmware_version = FirmwareVersion { major, minor, patch, year };
        Ok(firmware_version)
    }
}

impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}.{}", self.major, self.minor, self.patch, self.year)
    }
}
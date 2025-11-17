use std::fmt;

use crate::iqos::error::{IQOSError, Result};

pub const LOAD_TELEMETRY_SIGNAL: [u8; 8] = [0x00, 0xC9, 0x10, 0x02, 0x01, 0x01, 0x75, 0xD6];
pub const LOAD_TIMESTAMP_SIGNAL: [u8; 8] = [0x00, 0xC0, 0x10, 0x02, 0x00, 0x04, 0x38, 0xEF];

pub struct Telemetry {
    total_usage_count: u16,
    usage_period: u8,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            total_usage_count: 0,
            usage_period: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes[2] != 0x90 || bytes[3] != 0x22 {
            return Err(IQOSError::ConfigurationError("Invalid Telemetry data".to_string()));
        }
 
        let total_usage_count = u16::from_le_bytes([bytes[10], bytes[11]]);
        println!("Total usage count bytes: {}", total_usage_count);

        Ok(Self {
            total_usage_count,
            usage_period: 0,
        })
    }
}

pub const LOAD_TOTAL_SMOKING_SIGNAL: [u8; 8] = [0x00, 0xC9, 0x10, 0x02, 0x01, 0x01, 0x75, 0xD6];
pub const LOAD_TIMESTAMP_SIGNAL: [u8; 8] = [0x00, 0xC0, 0x10, 0x02, 0x00, 0x04, 0x38, 0xEF];
pub const LOAD_USAGE_COUNT_HEATING_TIME_SIGNAL: [u8; 8] = [0x00, 0xC9, 0x10, 0x02, 0x01, 0x01, 0x75, 0xD6];
pub const LOAD_BATTERY_VOLTAGE_SIGNAL: [u8; 5] = [0x00, 0xC0, 0x00, 0x21, 0xE7];

pub const ALL_DIAGNOSIS_SIGNALS: &[&[u8]] = &[
    &LOAD_TOTAL_SMOKING_SIGNAL,
    &LOAD_TIMESTAMP_SIGNAL,
    &LOAD_USAGE_COUNT_HEATING_TIME_SIGNAL,
    &LOAD_BATTERY_VOLTAGE_SIGNAL,
];

pub const TOTAL_SMOKING_HEADER: [u8; 2] = [0x90, 0x22];
pub const TIMESTAMP_HEADER: [u8; 2] = [0x80, 0x02]; // Example - adjust to actual
pub const BATTERY_VOLTAGE_HEADER: [u8; 2] = [0x88, 0x21]; // Example - adjust to actual

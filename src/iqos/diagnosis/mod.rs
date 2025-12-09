mod diagnosis;
mod builder;
mod constants;

pub use diagnosis::{
    Diagnosis,
    DiagnosisCollector,
    ParseIntoDiagnosis,
};

pub use builder::{
    DiagnosisBuilder
};

pub use constants::{
    LOAD_TOTAL_SMOKING_SIGNAL,
    LOAD_TIMESTAMP_SIGNAL,
    LOAD_USAGE_COUNT_HEATING_TIME_SIGNAL,
    LOAD_BATTERY_VOLTAGE_SIGNAL,
    ALL_DIAGNOSIS_SIGNALS,
};

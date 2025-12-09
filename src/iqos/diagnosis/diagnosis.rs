use std::fmt;

use crate::iqos::error::{IQOSError, Result};
use crate::iqos::diagnosis::builder::DiagnosisBuilder;

#[derive(Default, Debug, Clone)]
pub struct Diagnosis {
    pub(super) total_smoking_count: Option<u16>,
    pub(super) session: Option<u8>,
    pub(super) days_used: Option<u16>,
    pub(super) heating_count: Option<u32>,
    pub(super) composite_counter: Option<u16>,
    pub(super) battery_voltage: Option<f32>,
    pub(super) usage_period: Option<u8>,
    pub(super) total_heating_time: Option<u32>,
}

impl Diagnosis {
    /// Creates a new empty `Diagnosis`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder for constructing `Diagnosis` incrementally.
    pub fn builder() -> DiagnosisBuilder {
        DiagnosisBuilder::new()
    }

    // ========================================================================
    // Getters
    // ========================================================================

    pub fn total_usage_count(&self) -> Option<u16> {
        self.total_smoking_count
    }

    pub fn session(&self) -> Option<u8> {
        self.session
    }

    pub fn days_used(&self) -> Option<u16> {
        self.days_used
    }

    pub fn heating_count(&self) -> Option<u32> {
        self.heating_count
    }

    pub fn composite_counter(&self) -> Option<u16> {
        self.composite_counter
    }

    pub fn battery_voltage(&self) -> Option<f32> {
        self.battery_voltage
    }

    pub fn usage_period(&self) -> Option<u8> {
        self.usage_period
    }

    pub fn total_heating_time(&self) -> Option<u32> {
        self.total_heating_time
    }

    /// Returns `true` if all required fields are populated.
    pub fn is_complete(&self) -> bool {
        self.total_smoking_count.is_some() && self.session.is_some()
    }
}

impl fmt::Display for Diagnosis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Diagnosis:")?;
        if let Some(v) = self.total_smoking_count {
            writeln!(f, "  Total usage count: {v}")?;
        }
        if let Some(v) = self.session {
            writeln!(f, "  Session count: {v}")?;
        }
        if let Some(v) = self.days_used {
            writeln!(f, "  Days used: {v}")?;
        }
        if let Some(v) = self.heating_count {
            writeln!(f, "  Heating count: {v}")?;
        }
        if let Some(v) = self.battery_voltage {
            writeln!(f, "  Battery voltage: {v:.2}V")?;
        }
        if let Some(v) = self.total_heating_time {
            writeln!(f, "  Total heating time: {v}s")?;
        }
        Ok(())
    }
}



// ============================================================================
// ParseDiagnosis trait - for extensible parsing
// ============================================================================

/// Trait for types that can contribute to `Diagnosis` construction.
/// This enables a plugin-like architecture for different response types.
pub trait ParseIntoDiagnosis {
    fn parse_into(self, builder: DiagnosisBuilder) -> Result<DiagnosisBuilder>;
}

impl ParseIntoDiagnosis for &[u8] {
    fn parse_into(self, builder: DiagnosisBuilder) -> Result<DiagnosisBuilder> {
        builder.parse(self)
    }
}

/// Extension trait for iterators to collect into a `Diagnosis`.
pub trait DiagnosisCollector: Iterator + Sized
where
    Self::Item: AsRef<[u8]>,
{
    /// Collects byte responses into a `Diagnosis`.
    /// 
    /// # Example
    /// ```ignore
    /// let diagnosis: Diagnosis = responses
    ///     .iter()
    ///     .collect_diagnosis()?;
    /// ```
    fn collect_diagnosis(self) -> Result<Diagnosis> {
        let mut builder = DiagnosisBuilder::new();
        for bytes in self {
            builder = builder.parse(bytes.as_ref())?;
        }
        Ok(builder.build())
    }
}

impl<I> DiagnosisCollector for I
where
    I: Iterator + Sized,
    I::Item: AsRef<[u8]>,
{
}
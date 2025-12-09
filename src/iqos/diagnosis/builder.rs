use crate::iqos::error::{IQOSError, Result};
use crate::iqos::diagnosis::Diagnosis;
use crate::iqos::diagnosis::constants::{
    TOTAL_SMOKING_HEADER,
    TIMESTAMP_HEADER,
    BATTERY_VOLTAGE_HEADER,
};

// ============================================================================
// DiagnosisBuilder - Fluent API for incremental construction
// ============================================================================

/// A builder that accumulates diagnosis data from multiple byte responses.
/// 
/// # Example
/// ```ignore
/// let diagnosis = Diagnosis::builder()
///     .parse(&telemetry_response)?
///     .parse(&timestamp_response)?
///     .parse(&usage_response)?
///     .build();
/// ```
#[derive(Default, Debug, Clone)]
pub struct DiagnosisBuilder {
    inner: Diagnosis,
}

impl DiagnosisBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a byte slice and updates the internal state based on the response header.
    /// This method auto-detects the response type and extracts relevant fields.
    /// 
    /// # Fluent API
    /// Returns `Result<Self>` for method chaining with `?` operator.
    pub fn parse(mut self, bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(IQOSError::ConfigurationError(
                "Response too short for diagnosis parsing".to_string(),
            ));
        }

        let header = [bytes[2], bytes[3]];

        match header {
            TOTAL_SMOKING_HEADER => self.parse_telemetry(bytes)?,
            TIMESTAMP_HEADER => self.parse_timestamp(bytes)?,
            BATTERY_VOLTAGE_HEADER => self.parse_battery_voltage(bytes)?,
            _ => {
                // Unknown header - try heuristic parsing or skip
                self.try_heuristic_parse(bytes)?;
            }
        }

        Ok(self)
    }

    /// Parses multiple byte responses at once.
    /// 
    /// # Example
    /// ```ignore
    /// let diagnosis = Diagnosis::builder()
    ///     .parse_all(&[&response1, &response2, &response3])?
    ///     .build();
    /// ```
    pub fn parse_all(self, responses: &[&[u8]]) -> Result<Self> {
        responses.iter().try_fold(self, |builder, bytes| builder.parse(bytes))
    }

    /// Finalizes the builder and returns the constructed `Diagnosis`.
    pub fn build(self) -> Diagnosis {
        self.inner
    }

    /// Finalizes the builder, returning an error if required fields are missing.
    pub fn build_complete(self) -> Result<Diagnosis> {
        if !self.inner.is_complete() {
            return Err(IQOSError::ConfigurationError(
                "Diagnosis is incomplete: missing required fields".to_string(),
            ));
        }
        Ok(self.inner)
    }

    // ========================================================================
    // Private parsing methods
    // ========================================================================

    fn parse_telemetry(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() < 15 {
            return Err(IQOSError::ConfigurationError(
                "Telemetry data too short".to_string(),
            ));
        }

        self.inner.total_smoking_count = Some(u16::from_le_bytes([bytes[10], bytes[11]]));
        self.inner.session = Some(u16::from_le_bytes([bytes[13], bytes[14]]) as u8);
        self.inner.composite_counter = Some(u16::from_le_bytes([bytes[26], bytes[27]]));
        self.inner.days_used = Some(u16::from_le_bytes([bytes[34], bytes[35]]));

        Ok(())
    }

    fn parse_timestamp(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() < 8 {
            return Err(IQOSError::ConfigurationError(
                "Timestamp data too short".to_string(),
            ));
        }

        // Extract timestamp-related fields
        self.inner.days_used = Some(u16::from_le_bytes([bytes[4], bytes[5]]));

        Ok(())
    }

    fn parse_battery_voltage(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() < 6 {
            return Err(IQOSError::ConfigurationError(
                "Battery voltage data too short".to_string(),
            ));
        }

        let raw_voltage = u16::from_le_bytes([bytes[5], bytes[6]]);
        self.inner.battery_voltage = Some(raw_voltage as f32 / 1000.0); // Assuming mV to V conversion

        Ok(())
    }

    fn try_heuristic_parse(&mut self, bytes: &[u8]) -> Result<()> {
        // Fallback parsing based on data patterns
        // This allows graceful handling of unknown but valid responses
        if bytes.len() >= 12 && bytes[0] == 0x00 {
            // Attempt to extract common patterns
            // Add specific heuristics as needed
        }
        Ok(())
    }

    // ========================================================================
    // Manual setters for edge cases
    // ========================================================================

    pub fn with_total_usage_count(mut self, value: u16) -> Self {
        self.inner.total_smoking_count = Some(value);
        self
    }

    pub fn with_session(mut self, value: u8) -> Self {
        self.inner.session = Some(value);
        self
    }

    pub fn with_days_used(mut self, value: u16) -> Self {
        self.inner.days_used = Some(value);
        self
    }

    pub fn with_heating_count(mut self, value: u32) -> Self {
        self.inner.heating_count = Some(value);
        self
    }

    pub fn with_battery_voltage(mut self, value: f32) -> Self {
        self.inner.battery_voltage = Some(value);
        self
    }

    pub fn with_total_heating_time(mut self, value: u32) -> Self {
        self.inner.total_heating_time = Some(value);
        self
    }
}
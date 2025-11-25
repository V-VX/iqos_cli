use super::variant::IlumaVibrationBehavior;
use super::iluma::IlumaVibration;

pub const LOAD_VIBRATION_SETTINGS_SIGNAL: [u8; 5] = [0x00, 0xc9, 0x00, 0x23, 0xE9];
pub const LOAD_VIBRATE_CHARGE_START_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08];

pub const WHEN_STARTING_TO_USE_SIGNAL: u16 = 0x1000;
pub const WHEN_HEATING_START_SIGNAL: u16 = 0x0100;
pub const WHEN_MANUALLY_TERMINATED_SIGNAL: u16 = 0x0010;
pub const WHEN_PUFF_END_SIGNAL: u16 = 0x0001;

#[derive(Debug, Clone)]
pub struct VibrationSettings {
    pub iluma_and_higher: Option<IlumaVibration>,
    pub when_heating_start: Option<bool>,
    pub when_starting_to_use: Option<bool>,
    pub when_puff_end: Option<bool>,
    pub when_manually_terminated: Option<bool>,
}

impl VibrationSettings {
    pub fn new(
        when_heating_start: bool,
        when_starting_to_use: bool,
        when_puff_end: bool,
        when_manually_terminated: bool,
    ) -> Self {
        Self {
            iluma_and_higher: None,
            when_heating_start: Some(when_heating_start),
            when_starting_to_use: Some(when_starting_to_use),
            when_puff_end: Some(when_puff_end),
            when_manually_terminated: Some(when_manually_terminated),
        }
    }

    pub fn is_iluma(&self) -> bool {
        self.iluma_and_higher.is_some()
    }

    pub fn with_iluma(
        when_heating_start: bool,
        when_starting_to_use: bool,
        when_puff_end: bool,
        when_manually_terminated: bool,
        when_charging_start: bool,
    ) -> Self {
        Self {
            iluma_and_higher: Some(IlumaVibration { when_charging_start }),
            when_heating_start: Some(when_heating_start),
            when_starting_to_use: Some(when_starting_to_use),
            when_puff_end: Some(when_puff_end),
            when_manually_terminated: Some(when_manually_terminated),
        }
    }
    
    pub fn when_heating_start(&self) -> bool {
        self.when_heating_start.unwrap_or(false)
    }
    
    pub fn when_starting_to_use(&self) -> bool {
        self.when_starting_to_use.unwrap_or(false)
    }
    
    pub fn when_puff_end(&self) -> bool {
        self.when_puff_end.unwrap_or(false)
    }
    
    pub fn when_manually_terminated(&self) -> bool {
        self.when_manually_terminated.unwrap_or(false)
    }
    
    pub fn when_charging_start(&self) -> bool {
        self.iluma_and_higher.as_ref().map_or(false, |iluma| iluma.when_charging_start)
    }

    /// Returns Some(&self) if this is an Iluma device's settings, None otherwise
    pub fn as_iluma(&self) -> Option<&impl IlumaVibrationBehavior> {
        self.iluma_and_higher.as_ref().map(|_| self)
    }
}

impl std::fmt::Display for VibrationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_iluma() {
            let charging_start = self.when_charging_start();
            write!(f, "\nVibration Settings\n\twhen charge start: {}\n\twhen heating: {}\n\twhen starting: {}\n\twhen puff end soon: {}\n\twhen terminated: {}\n",
                charging_start,
                self.when_heating_start(),
                self.when_starting_to_use(),
                self.when_puff_end(),
                self.when_manually_terminated(),
            )
        } else {
            write!(f, "\nVibration Settings\n\twhen heating: {}\n\twhen starting: {}\n\twhen puff end soon: {}\n\twhen terminated: {}\n",
                self.when_heating_start(),
                self.when_starting_to_use(),
                self.when_puff_end(),
                self.when_manually_terminated(),
            )
        }
    }
}
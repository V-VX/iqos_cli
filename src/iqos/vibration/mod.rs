mod settings;
mod variant;
mod iluma;
mod iluma_one;

pub use variant::VibrationBehavior;
pub use variant::IlumaVibrationBehavior;

pub use settings::VibrationSettings;
pub use settings::{LOAD_VIBRATION_SETTINGS_SIGNAL, LOAD_VIBRATE_CHARGE_START_SIGNAL};
pub use iluma::IlumaVibration;
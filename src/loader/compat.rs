// This module intentionally diverges from the iqos crate's DeviceCapability matrix
// where device-testing has revealed more accurate per-feature gates.
// Re-evaluate each function below when the crate updates its capability matrix.

use iqos::DeviceModel;

pub const fn supports_brightness(model: DeviceModel) -> bool {
    matches!(
        model,
        DeviceModel::IlumaOne
            | DeviceModel::Iluma
            | DeviceModel::IlumaPrime
            | DeviceModel::IlumaIOne
            | DeviceModel::IlumaI
            | DeviceModel::IlumaIPrime
    )
}

pub const fn supports_flexbattery(model: DeviceModel) -> bool {
    matches!(model, DeviceModel::IlumaI | DeviceModel::IlumaIPrime)
}

// Crate v1.1.0 maps FlexPuff to supports_holder_features() (Iluma, IlumaPrime included).
// FlexPuff requires the ILUMA i gesture sensor — iluma_i_family() is intentionally narrower.
pub const fn supports_flexpuff(model: DeviceModel) -> bool {
    model.is_iluma_i_family()
}

// Crate v1.1.0 maps SmartGesture to supports_holder_features() (Iluma, IlumaPrime included).
// SmartGesture requires the ILUMA i gesture sensor — iluma_i_family() is intentionally narrower.
pub const fn supports_smartgesture(model: DeviceModel) -> bool {
    model.is_iluma_i_family()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_supports_all_known_iluma_models() {
        for model in [
            DeviceModel::IlumaOne,
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaIOne,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(supports_brightness(model), "{model:?}");
        }

        assert!(!supports_brightness(DeviceModel::Unknown));
    }

    #[test]
    fn flexbattery_supports_iluma_i_holder_models_only() {
        assert!(supports_flexbattery(DeviceModel::IlumaI));
        assert!(supports_flexbattery(DeviceModel::IlumaIPrime));
        assert!(!supports_flexbattery(DeviceModel::IlumaIOne));
        assert!(!supports_flexbattery(DeviceModel::Iluma));
        assert!(!supports_flexbattery(DeviceModel::IlumaPrime));
    }

    #[test]
    fn flexpuff_and_smartgesture_require_iluma_i_gesture_sensor() {
        for model in [DeviceModel::IlumaIOne, DeviceModel::IlumaI, DeviceModel::IlumaIPrime] {
            assert!(supports_flexpuff(model), "{model:?}");
            assert!(supports_smartgesture(model), "{model:?}");
        }

        for model in [DeviceModel::Iluma, DeviceModel::IlumaPrime, DeviceModel::IlumaOne] {
            assert!(!supports_flexpuff(model), "{model:?}");
            assert!(!supports_smartgesture(model), "{model:?}");
        }
    }
}

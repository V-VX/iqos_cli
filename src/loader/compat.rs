// Keep CLI feature gates aligned with the iqos crate. The command handlers call
// crate APIs that enforce the same matrix, so divergence would produce commands
// that appear available but fail after dispatch.

use iqos::{DeviceCapability, DeviceModel};

pub const fn supports_brightness(model: DeviceModel) -> bool {
    model.supports(DeviceCapability::Brightness)
}

pub const fn supports_flexbattery(model: DeviceModel) -> bool {
    model.supports(DeviceCapability::FlexBattery)
}

pub const fn supports_flexpuff(model: DeviceModel) -> bool {
    model.supports(DeviceCapability::FlexPuff)
}

pub const fn supports_smartgesture(model: DeviceModel) -> bool {
    model.supports(DeviceCapability::SmartGesture)
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
    fn flexpuff_requires_iluma_i_holder_models() {
        for model in [DeviceModel::IlumaI, DeviceModel::IlumaIPrime] {
            assert!(supports_flexpuff(model), "{model:?}");
        }

        for model in [
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaOne,
            DeviceModel::IlumaIOne,
        ] {
            assert!(!supports_flexpuff(model), "{model:?}");
        }
    }

    #[test]
    fn smartgesture_supports_holder_models() {
        for model in [
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaIOne,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(supports_smartgesture(model), "{model:?}");
        }

        for model in [DeviceModel::IlumaOne] {
            assert!(!supports_smartgesture(model), "{model:?}");
        }
    }
}

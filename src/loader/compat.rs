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

pub const fn supports_flexpuff(model: DeviceModel) -> bool {
    model.is_iluma_i_family()
}

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
    fn flexbattery_supports_iluma_i_holder_models() {
        assert!(supports_flexbattery(DeviceModel::IlumaI));
        assert!(supports_flexbattery(DeviceModel::IlumaIPrime));
        assert!(!supports_flexbattery(DeviceModel::IlumaIOne));
        assert!(!supports_flexbattery(DeviceModel::Iluma));
    }

    #[test]
    fn flexpuff_and_smartgesture_support_iluma_i_series() {
        for model in [
            DeviceModel::IlumaIOne,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(supports_flexpuff(model), "{model:?}");
            assert!(supports_smartgesture(model), "{model:?}");
        }

        assert!(!supports_flexpuff(DeviceModel::Iluma));
        assert!(!supports_smartgesture(DeviceModel::Iluma));
    }
}

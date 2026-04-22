use iqos::DeviceModel;

pub fn parse_device_model(value: &str) -> Option<DeviceModel> {
    let normalized = value.trim().to_ascii_lowercase().replace(['_', ' '], "-");

    match normalized.as_str() {
        "iluma" => Some(DeviceModel::Iluma),
        "iluma-prime" => Some(DeviceModel::IlumaPrime),
        "iluma-one" => Some(DeviceModel::IlumaOne),
        "iluma-i" => Some(DeviceModel::IlumaI),
        "iluma-i-prime" => Some(DeviceModel::IlumaIPrime),
        "iluma-i-one" => Some(DeviceModel::IlumaIOne),
        _ => None,
    }
}

pub fn is_reserved_model_label(value: &str) -> bool {
    parse_device_model(value).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_model_selectors() {
        assert_eq!(parse_device_model("iluma"), Some(DeviceModel::Iluma));
        assert_eq!(
            parse_device_model("ILUMA-I-PRIME"),
            Some(DeviceModel::IlumaIPrime)
        );
        assert_eq!(
            parse_device_model("iluma_i_one"),
            Some(DeviceModel::IlumaIOne)
        );
    }

    #[test]
    fn recognizes_reserved_model_labels() {
        assert!(is_reserved_model_label("iluma i"));
        assert!(is_reserved_model_label("ILUMA_I_PRIME"));
        assert!(!is_reserved_model_label("minera"));
    }
}

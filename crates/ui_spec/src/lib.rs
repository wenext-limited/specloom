#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiSpec {
    pub spec_version: String,
}

impl Default for UiSpec {
    fn default() -> Self {
        Self {
            spec_version: "1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_spec_round_trip() {
        let spec = UiSpec::default();
        let json = serde_json::to_string(&spec).unwrap();
        let back: UiSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }
}

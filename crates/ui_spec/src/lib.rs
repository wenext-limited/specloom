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

impl UiSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
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

    #[test]
    fn serialization_is_stable() {
        let spec = UiSpec::new();
        let a = spec.to_pretty_json().unwrap();
        let b = spec.to_pretty_json().unwrap();
        assert_eq!(a, b);
    }
}

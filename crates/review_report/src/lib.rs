#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReviewReport {
    pub report_version: String,
}

impl Default for ReviewReport {
    fn default() -> Self {
        Self {
            report_version: "1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_report_round_trip() {
        let report = ReviewReport::default();
        let json = serde_json::to_string(&report).unwrap();
        let back: ReviewReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
    }
}

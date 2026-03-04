#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReviewReport {
    pub report_version: String,
    pub warnings: Vec<ReviewWarning>,
    pub summary: ReviewSummary,
}

impl Default for ReviewReport {
    fn default() -> Self {
        Self::from_warnings(Vec::new())
    }
}

impl ReviewReport {
    pub fn from_warnings(warnings: Vec<ReviewWarning>) -> Self {
        let summary = ReviewSummary::from_warnings(&warnings);
        Self {
            report_version: "1.0".to_string(),
            warnings,
            summary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReviewWarning {
    pub code: String,
    pub category: ReviewWarningCategory,
    pub severity: ReviewWarningSeverity,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewWarningCategory {
    UnsupportedFeature,
    LowConfidenceLayout,
    FallbackApplied,
    DataLossRisk,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewWarningSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReviewSummary {
    pub total_warnings: usize,
    pub by_category: BTreeMap<ReviewWarningCategory, usize>,
    pub by_severity: BTreeMap<ReviewWarningSeverity, usize>,
}

impl ReviewSummary {
    fn from_warnings(warnings: &[ReviewWarning]) -> Self {
        let mut by_category = BTreeMap::new();
        let mut by_severity = BTreeMap::new();

        for warning in warnings {
            *by_category.entry(warning.category.clone()).or_insert(0) += 1;
            *by_severity.entry(warning.severity.clone()).or_insert(0) += 1;
        }

        Self {
            total_warnings: warnings.len(),
            by_category,
            by_severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_report_round_trip() {
        let report = ReviewReport::from_warnings(vec![
            sample_warning(
                "UNSUPPORTED_MASK",
                ReviewWarningCategory::UnsupportedFeature,
                ReviewWarningSeverity::High,
                Some("22:9"),
            ),
            sample_warning(
                "LOW_CONFIDENCE_LAYOUT",
                ReviewWarningCategory::LowConfidenceLayout,
                ReviewWarningSeverity::Medium,
                Some("18:3"),
            ),
        ]);
        let json = serde_json::to_string(&report).unwrap();
        let back: ReviewReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
    }

    #[test]
    fn summary_counters_are_classified() {
        let report = ReviewReport::from_warnings(vec![
            sample_warning(
                "UNSUPPORTED_MASK",
                ReviewWarningCategory::UnsupportedFeature,
                ReviewWarningSeverity::High,
                Some("22:9"),
            ),
            sample_warning(
                "LOW_CONFIDENCE_LAYOUT",
                ReviewWarningCategory::LowConfidenceLayout,
                ReviewWarningSeverity::Medium,
                Some("18:3"),
            ),
            sample_warning(
                "LOW_CONFIDENCE_LAYOUT_2",
                ReviewWarningCategory::LowConfidenceLayout,
                ReviewWarningSeverity::Medium,
                Some("18:4"),
            ),
        ]);

        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::UnsupportedFeature),
            Some(&1)
        );
        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::LowConfidenceLayout),
            Some(&2)
        );
        assert_eq!(
            report
                .summary
                .by_severity
                .get(&ReviewWarningSeverity::Medium),
            Some(&2)
        );
    }

    fn sample_warning(
        code: &str,
        category: ReviewWarningCategory,
        severity: ReviewWarningSeverity,
        node_id: Option<&str>,
    ) -> ReviewWarning {
        ReviewWarning {
            code: code.to_string(),
            category,
            severity,
            message: "example warning".to_string(),
            node_id: node_id.map(str::to_string),
        }
    }
}

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

pub fn map_inference_warnings(warnings: &[layout_infer::InferenceWarning]) -> Vec<ReviewWarning> {
    warnings
        .iter()
        .map(|warning| ReviewWarning {
            code: warning.code.clone(),
            category: map_inference_category(warning),
            severity: map_inference_severity(warning.severity.clone()),
            message: warning.message.clone(),
            node_id: warning.node_id.clone(),
        })
        .collect()
}

fn map_inference_category(warning: &layout_infer::InferenceWarning) -> ReviewWarningCategory {
    match layout_infer::warning_kind(warning.code.as_str()) {
        layout_infer::InferenceWarningKind::LowConfidence => {
            ReviewWarningCategory::LowConfidenceLayout
        }
        layout_infer::InferenceWarningKind::UnsupportedFeature => {
            ReviewWarningCategory::UnsupportedFeature
        }
        layout_infer::InferenceWarningKind::FallbackApplied => {
            ReviewWarningCategory::FallbackApplied
        }
    }
}

fn map_inference_severity(severity: layout_infer::WarningSeverity) -> ReviewWarningSeverity {
    match severity {
        layout_infer::WarningSeverity::Low => ReviewWarningSeverity::Low,
        layout_infer::WarningSeverity::Medium => ReviewWarningSeverity::Medium,
        layout_infer::WarningSeverity::High => ReviewWarningSeverity::High,
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
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewWarningCategory {
    UnsupportedFeature,
    LowConfidenceLayout,
    FallbackApplied,
    DataLossRisk,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewWarningSeverity {
    Low,
    Medium,
    High,
}

impl ReviewWarningCategory {
    pub const ALL: [Self; 4] = [
        Self::UnsupportedFeature,
        Self::LowConfidenceLayout,
        Self::FallbackApplied,
        Self::DataLossRisk,
    ];
}

impl ReviewWarningSeverity {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReviewSummary {
    pub total_warnings: usize,
    pub by_category: BTreeMap<ReviewWarningCategory, usize>,
    pub by_severity: BTreeMap<ReviewWarningSeverity, usize>,
}

impl ReviewSummary {
    fn from_warnings(warnings: &[ReviewWarning]) -> Self {
        let mut by_category = ReviewWarningCategory::ALL
            .into_iter()
            .map(|category| (category, 0))
            .collect::<BTreeMap<_, _>>();
        let mut by_severity = ReviewWarningSeverity::ALL
            .into_iter()
            .map(|severity| (severity, 0))
            .collect::<BTreeMap<_, _>>();

        for warning in warnings {
            let category_count = by_category
                .get_mut(&warning.category)
                .expect("every warning category must be initialized");
            *category_count += 1;
            let severity_count = by_severity
                .get_mut(&warning.severity)
                .expect("every warning severity must be initialized");
            *severity_count += 1;
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

    #[test]
    fn summary_includes_zero_counts_for_all_categories_and_severities() {
        let report = ReviewReport::from_warnings(vec![sample_warning(
            "LOW_CONFIDENCE_LAYOUT",
            ReviewWarningCategory::LowConfidenceLayout,
            ReviewWarningSeverity::Medium,
            Some("18:3"),
        )]);

        assert_eq!(report.summary.by_category.len(), 4);
        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::UnsupportedFeature),
            Some(&0)
        );
        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::LowConfidenceLayout),
            Some(&1)
        );
        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::FallbackApplied),
            Some(&0)
        );
        assert_eq!(
            report
                .summary
                .by_category
                .get(&ReviewWarningCategory::DataLossRisk),
            Some(&0)
        );

        assert_eq!(report.summary.by_severity.len(), 3);
        assert_eq!(
            report.summary.by_severity.get(&ReviewWarningSeverity::Low),
            Some(&0)
        );
        assert_eq!(
            report
                .summary
                .by_severity
                .get(&ReviewWarningSeverity::Medium),
            Some(&1)
        );
        assert_eq!(
            report.summary.by_severity.get(&ReviewWarningSeverity::High),
            Some(&0)
        );
    }

    #[test]
    fn warning_contract_values_are_stable_snake_case() {
        let category_json =
            serde_json::to_string(&ReviewWarningCategory::LowConfidenceLayout).unwrap();
        assert_eq!(category_json, "\"low_confidence_layout\"");
        let category: ReviewWarningCategory = serde_json::from_str(&category_json).unwrap();
        assert_eq!(category, ReviewWarningCategory::LowConfidenceLayout);

        let severity_json = serde_json::to_string(&ReviewWarningSeverity::Medium).unwrap();
        assert_eq!(severity_json, "\"medium\"");
        let severity: ReviewWarningSeverity = serde_json::from_str(&severity_json).unwrap();
        assert_eq!(severity, ReviewWarningSeverity::Medium);
    }

    #[test]
    fn maps_inference_warnings_into_review_warnings() {
        let inference_warnings = vec![
            layout_infer::InferenceWarning {
                code: layout_infer::WARNING_LOW_CONFIDENCE_GEOMETRY.to_string(),
                severity: layout_infer::WarningSeverity::Medium,
                message: "Ambiguous geometry".to_string(),
                node_id: Some("1:1".to_string()),
            },
            layout_infer::InferenceWarning {
                code: layout_infer::WARNING_UNSUPPORTED_NODE_KIND.to_string(),
                severity: layout_infer::WarningSeverity::High,
                message: "Unsupported node kind".to_string(),
                node_id: Some("2:2".to_string()),
            },
        ];

        let mapped = super::map_inference_warnings(&inference_warnings);
        assert_eq!(mapped.len(), 2);
        assert_eq!(
            mapped[0].category,
            ReviewWarningCategory::LowConfidenceLayout
        );
        assert_eq!(mapped[0].severity, ReviewWarningSeverity::Medium);
        assert_eq!(
            mapped[1].category,
            ReviewWarningCategory::UnsupportedFeature
        );
        assert_eq!(mapped[1].severity, ReviewWarningSeverity::High);
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

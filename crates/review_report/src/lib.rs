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

pub fn build_review_report(
    normalization_warnings: &[figma_normalizer::NormalizationWarning],
    inferred_layout: &layout_infer::InferredLayoutDocument,
    asset_warnings: &[asset_pipeline::AssetExportWarning],
) -> ReviewReport {
    let mut warnings = Vec::new();
    warnings.extend(map_normalization_warnings(normalization_warnings));

    let inferred_warnings = inferred_layout
        .decisions
        .iter()
        .flat_map(|decision| decision.record.warnings.iter().cloned())
        .collect::<Vec<_>>();
    warnings.extend(map_inference_warnings(&inferred_warnings));

    warnings.extend(map_asset_warnings(asset_warnings));
    ReviewReport::from_warnings(warnings)
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

fn map_normalization_warnings(
    warnings: &[figma_normalizer::NormalizationWarning],
) -> Vec<ReviewWarning> {
    warnings
        .iter()
        .map(|warning| ReviewWarning {
            code: warning.code.clone(),
            category: map_structured_warning_category(warning.code.as_str()),
            severity: map_structured_warning_severity(warning.code.as_str()),
            message: warning.message.clone(),
            node_id: warning.node_id.clone(),
        })
        .collect()
}

fn map_asset_warnings(warnings: &[asset_pipeline::AssetExportWarning]) -> Vec<ReviewWarning> {
    warnings
        .iter()
        .map(|warning| {
            let (category, severity) = if warning.fallback_applied {
                (
                    ReviewWarningCategory::FallbackApplied,
                    ReviewWarningSeverity::Medium,
                )
            } else {
                (
                    map_structured_warning_category(warning.code.as_str()),
                    map_structured_warning_severity(warning.code.as_str()),
                )
            };

            ReviewWarning {
                code: warning.code.clone(),
                category,
                severity,
                message: warning.message.clone(),
                node_id: warning.node_id.clone(),
            }
        })
        .collect()
}

fn map_structured_warning_category(code: &str) -> ReviewWarningCategory {
    if code.starts_with("UNSUPPORTED_") {
        ReviewWarningCategory::UnsupportedFeature
    } else if code.starts_with("LOW_CONFIDENCE_") {
        ReviewWarningCategory::LowConfidenceLayout
    } else if code.contains("FALLBACK") {
        ReviewWarningCategory::FallbackApplied
    } else {
        ReviewWarningCategory::DataLossRisk
    }
}

fn map_structured_warning_severity(code: &str) -> ReviewWarningSeverity {
    if code.starts_with("UNSUPPORTED_") || code.starts_with("MISSING_") {
        ReviewWarningSeverity::High
    } else if code.starts_with("LOW_CONFIDENCE_") || code.contains("FALLBACK") {
        ReviewWarningSeverity::Medium
    } else {
        ReviewWarningSeverity::Low
    }
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

    #[test]
    fn build_review_report_aggregates_warnings_from_all_stage_sources() {
        let normalization_warnings = vec![figma_normalizer::NormalizationWarning {
            code: "UNSUPPORTED_NODE_FIELD".to_string(),
            message: "unsupported field `clipsContent` ignored during normalization".to_string(),
            node_id: Some("1:1".to_string()),
        }];
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: vec![layout_infer::NodeLayoutDecision {
                node_id: "1:1".to_string(),
                record: layout_infer::LayoutDecisionRecord {
                    decision_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
                    selected_strategy: layout_infer::LayoutStrategy::VStack,
                    confidence: 0.62,
                    rationale: "ambiguous child geometry".to_string(),
                    alternatives: Vec::new(),
                    warnings: vec![layout_infer::InferenceWarning {
                        code: layout_infer::WARNING_LOW_CONFIDENCE_GEOMETRY.to_string(),
                        severity: layout_infer::WarningSeverity::Medium,
                        message: "Ambiguous geometry".to_string(),
                        node_id: Some("1:1".to_string()),
                    }],
                },
            }],
        };
        let asset_warnings = vec![asset_pipeline::AssetExportWarning {
            code: "MISSING_IMAGE_REF".to_string(),
            message: "Image fill had no image_ref and was skipped.".to_string(),
            node_id: Some("2:2".to_string()),
            fallback_applied: false,
        }];

        let report =
            super::build_review_report(&normalization_warnings, &inferred, &asset_warnings);

        assert_eq!(
            report
                .warnings
                .iter()
                .map(|warning| warning.code.clone())
                .collect::<Vec<_>>(),
            vec![
                "UNSUPPORTED_NODE_FIELD".to_string(),
                "LOW_CONFIDENCE_GEOMETRY".to_string(),
                "MISSING_IMAGE_REF".to_string(),
            ]
        );
        assert_eq!(
            report.warnings[0].category,
            ReviewWarningCategory::UnsupportedFeature
        );
        assert_eq!(
            report.warnings[1].category,
            ReviewWarningCategory::LowConfidenceLayout
        );
        assert_eq!(
            report.warnings[2].category,
            ReviewWarningCategory::DataLossRisk
        );
        assert_eq!(report.summary.total_warnings, 3);
    }

    #[test]
    fn build_review_report_maps_asset_fallback_warning() {
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };
        let asset_warnings = vec![asset_pipeline::AssetExportWarning {
            code: "FORMAT_FALLBACK".to_string(),
            message: "SVG export unavailable; fell back to PDF.".to_string(),
            node_id: Some("2:2".to_string()),
            fallback_applied: true,
        }];

        let report = super::build_review_report(&Vec::new(), &inferred, &asset_warnings);

        assert_eq!(report.warnings.len(), 1);
        assert_eq!(
            report.warnings[0].category,
            ReviewWarningCategory::FallbackApplied
        );
        assert_eq!(report.warnings[0].severity, ReviewWarningSeverity::Medium);
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

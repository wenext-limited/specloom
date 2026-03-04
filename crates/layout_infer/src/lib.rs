#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutDecisionRecord {
    pub decision_version: String,
    pub selected_strategy: LayoutStrategy,
    pub confidence: f32,
    pub rationale: String,
    pub alternatives: Vec<LayoutAlternative>,
    pub warnings: Vec<InferenceWarning>,
}

impl Default for LayoutDecisionRecord {
    fn default() -> Self {
        Self {
            decision_version: "1.0".to_string(),
            selected_strategy: LayoutStrategy::Absolute,
            confidence: 0.0,
            rationale: String::new(),
            alternatives: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutAlternative {
    pub strategy: LayoutStrategy,
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceWarning {
    pub code: String,
    pub severity: WarningSeverity,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutStrategy {
    VStack,
    HStack,
    Overlay,
    Absolute,
    Scroll,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn layout_decision_round_trip() {
        let record = sample_record();
        let json = serde_json::to_string_pretty(&record).unwrap();
        let back: LayoutDecisionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, back);
    }

    #[test]
    fn alternatives_order_is_stable() {
        let record = sample_record();
        let json = serde_json::to_string_pretty(&record).unwrap();
        let back: LayoutDecisionRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(
            back.alternatives
                .iter()
                .map(|alt| alt.strategy.clone())
                .collect::<Vec<_>>(),
            vec![LayoutStrategy::Overlay, LayoutStrategy::Absolute]
        );
    }

    #[test]
    fn warnings_order_is_stable() {
        let record = LayoutDecisionRecord {
            warnings: vec![
                InferenceWarning {
                    code: "FIRST".to_string(),
                    severity: WarningSeverity::Low,
                    message: "First warning.".to_string(),
                    node_id: Some("1:1".to_string()),
                },
                InferenceWarning {
                    code: "SECOND".to_string(),
                    severity: WarningSeverity::High,
                    message: "Second warning.".to_string(),
                    node_id: Some("1:2".to_string()),
                },
            ],
            ..sample_record()
        };
        let json = serde_json::to_string_pretty(&record).unwrap();
        let back: LayoutDecisionRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(
            back.warnings
                .iter()
                .map(|warning| warning.code.clone())
                .collect::<Vec<_>>(),
            vec!["FIRST".to_string(), "SECOND".to_string()]
        );
    }

    #[test]
    fn decision_contract_fields_are_explicit_and_ordered() {
        let json = serde_json::to_string(&sample_record()).unwrap();

        assert_json_fields_in_order(
            &json,
            &[
                "\"decision_version\"",
                "\"selected_strategy\"",
                "\"confidence\"",
                "\"rationale\"",
                "\"alternatives\"",
                "\"warnings\"",
            ],
        );
    }

    #[test]
    fn warning_shape_is_explicit() {
        let warning = InferenceWarning {
            code: "AMBIGUOUS_LAYOUT".to_string(),
            severity: WarningSeverity::Medium,
            message: "Detected mixed layout signals.".to_string(),
            node_id: None,
        };

        let value = serde_json::to_value(warning).unwrap();
        assert_eq!(
            value,
            json!({
                "code": "AMBIGUOUS_LAYOUT",
                "severity": "medium",
                "message": "Detected mixed layout signals.",
                "node_id": null,
            })
        );
    }

    #[test]
    fn decision_record_rejects_unknown_fields() {
        let json = r#"{
            "decision_version":"1.0",
            "selected_strategy":"v_stack",
            "confidence":0.92,
            "rationale":"Primary axis and child spacing match vertical flow.",
            "alternatives":[],
            "warnings":[],
            "unexpected":"extra"
        }"#;

        let result = serde_json::from_str::<LayoutDecisionRecord>(json);
        assert!(result.is_err(), "unexpected fields must be rejected");
    }

    fn sample_record() -> LayoutDecisionRecord {
        LayoutDecisionRecord {
            decision_version: "1.0".to_string(),
            selected_strategy: LayoutStrategy::VStack,
            confidence: 0.92,
            rationale: "Primary axis and child spacing match vertical flow.".to_string(),
            alternatives: vec![
                LayoutAlternative {
                    strategy: LayoutStrategy::Overlay,
                    score: 0.43,
                    rationale: "Children overlap only partially.".to_string(),
                },
                LayoutAlternative {
                    strategy: LayoutStrategy::Absolute,
                    score: 0.12,
                    rationale: "Absolute placement loses auto layout intent.".to_string(),
                },
            ],
            warnings: vec![InferenceWarning {
                code: "LOW_CONFIDENCE_CHILD".to_string(),
                severity: WarningSeverity::Low,
                message: "One child has mixed constraints.".to_string(),
                node_id: Some("4:12".to_string()),
            }],
        }
    }

    fn assert_json_fields_in_order(json: &str, fields: &[&str]) {
        let mut next_index = 0;
        for field in fields {
            let found_index = json[next_index..]
                .find(field)
                .map(|offset| next_index + offset)
                .unwrap_or_else(|| panic!("field {field} not found in {json}"));
            assert!(
                found_index >= next_index,
                "field {field} appeared out of order in {json}"
            );
            next_index = found_index + field.len();
        }
    }
}

#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
pub struct LayoutAlternative {
    pub strategy: LayoutStrategy,
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
}

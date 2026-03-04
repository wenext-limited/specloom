#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub const LAYOUT_DECISION_VERSION: &str = "1.0";
pub const WARNING_LOW_CONFIDENCE_GEOMETRY: &str = "LOW_CONFIDENCE_GEOMETRY";
pub const WARNING_UNSUPPORTED_NODE_KIND: &str = "UNSUPPORTED_NODE_KIND";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceWarningKind {
    LowConfidence,
    UnsupportedFeature,
    FallbackApplied,
}

pub fn warning_kind(code: &str) -> InferenceWarningKind {
    match code {
        WARNING_LOW_CONFIDENCE_GEOMETRY => InferenceWarningKind::LowConfidence,
        WARNING_UNSUPPORTED_NODE_KIND => InferenceWarningKind::UnsupportedFeature,
        _ => InferenceWarningKind::FallbackApplied,
    }
}

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
            decision_version: LAYOUT_DECISION_VERSION.to_string(),
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
pub struct NodeLayoutDecision {
    pub node_id: String,
    pub record: LayoutDecisionRecord,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferredLayoutDocument {
    pub inference_version: String,
    pub source_file_key: String,
    pub root_node_id: String,
    pub decisions: Vec<NodeLayoutDecision>,
}

pub fn infer_layout(document: &figma_normalizer::NormalizedDocument) -> InferredLayoutDocument {
    let nodes_by_id = document
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    let decisions = document
        .nodes
        .iter()
        .map(|node| NodeLayoutDecision {
            node_id: node.id.clone(),
            record: infer_node_layout(node, &nodes_by_id),
        })
        .collect::<Vec<_>>();

    InferredLayoutDocument {
        inference_version: LAYOUT_DECISION_VERSION.to_string(),
        source_file_key: document.source.file_key.clone(),
        root_node_id: document.source.root_node_id.clone(),
        decisions,
    }
}

fn infer_node_layout(
    node: &figma_normalizer::NormalizedNode,
    nodes_by_id: &BTreeMap<&str, &figma_normalizer::NormalizedNode>,
) -> LayoutDecisionRecord {
    if node.children.is_empty() {
        return decision(
            LayoutStrategy::Absolute,
            1.0,
            "Leaf node has no children and keeps absolute placement.",
            Vec::new(),
            Vec::new(),
        );
    }

    if let Some(layout) = &node.layout {
        match layout.mode {
            figma_normalizer::LayoutMode::Vertical => {
                return decision(
                    LayoutStrategy::VStack,
                    0.98,
                    "Node uses explicit vertical auto layout metadata.",
                    vec![
                        alternative(
                            LayoutStrategy::Overlay,
                            0.41,
                            "Children could overlap but auto layout metadata takes precedence.",
                        ),
                        alternative(
                            LayoutStrategy::Absolute,
                            0.09,
                            "Absolute positioning would discard explicit auto layout intent.",
                        ),
                    ],
                    Vec::new(),
                );
            }
            figma_normalizer::LayoutMode::Horizontal => {
                return decision(
                    LayoutStrategy::HStack,
                    0.98,
                    "Node uses explicit horizontal auto layout metadata.",
                    vec![
                        alternative(
                            LayoutStrategy::Overlay,
                            0.38,
                            "Children could overlap but auto layout metadata takes precedence.",
                        ),
                        alternative(
                            LayoutStrategy::Absolute,
                            0.08,
                            "Absolute positioning would discard explicit auto layout intent.",
                        ),
                    ],
                    Vec::new(),
                );
            }
            figma_normalizer::LayoutMode::None => {}
        }
    }

    if matches!(node.kind, figma_normalizer::NodeKind::Unknown) {
        return decision(
            LayoutStrategy::Absolute,
            0.55,
            "Unknown node kind falls back to absolute placement.",
            vec![
                alternative(
                    LayoutStrategy::Overlay,
                    0.44,
                    "Unknown node type could represent an overlay container.",
                ),
                alternative(
                    LayoutStrategy::VStack,
                    0.26,
                    "Unknown node type could still be stack-like.",
                ),
            ],
            vec![InferenceWarning {
                code: WARNING_UNSUPPORTED_NODE_KIND.to_string(),
                severity: WarningSeverity::Medium,
                message: "Unsupported node kind forced absolute fallback.".to_string(),
                node_id: Some(node.id.clone()),
            }],
        );
    }

    infer_from_geometry(node, nodes_by_id)
}

fn infer_from_geometry(
    node: &figma_normalizer::NormalizedNode,
    nodes_by_id: &BTreeMap<&str, &figma_normalizer::NormalizedNode>,
) -> LayoutDecisionRecord {
    let children = node
        .children
        .iter()
        .filter_map(|id| nodes_by_id.get(id.as_str()).copied())
        .collect::<Vec<_>>();

    if children.len() <= 1 {
        return decision(
            LayoutStrategy::Overlay,
            0.85,
            "Single-child container defaults to overlay semantics.",
            vec![
                alternative(
                    LayoutStrategy::Absolute,
                    0.64,
                    "Absolute placement is viable but less reusable.",
                ),
                alternative(
                    LayoutStrategy::VStack,
                    0.21,
                    "Single child provides insufficient evidence for stack flow.",
                ),
            ],
            Vec::new(),
        );
    }

    if has_overlaps(&children) {
        return decision(
            LayoutStrategy::Overlay,
            0.86,
            "Child bounds overlap, indicating overlay composition.",
            vec![
                alternative(
                    LayoutStrategy::Absolute,
                    0.51,
                    "Absolute placement can represent overlap but is less semantic.",
                ),
                alternative(
                    LayoutStrategy::VStack,
                    0.18,
                    "Stack layout cannot represent heavy overlap reliably.",
                ),
            ],
            Vec::new(),
        );
    }

    let axis = dominant_axis(&children);
    if axis == DominantAxis::Vertical {
        return decision(
            LayoutStrategy::VStack,
            0.82,
            "Children are vertically distributed with aligned x positions.",
            vec![
                alternative(
                    LayoutStrategy::Absolute,
                    0.33,
                    "Absolute layout remains possible but less structured.",
                ),
                alternative(
                    LayoutStrategy::Overlay,
                    0.16,
                    "No overlap observed, so overlay is unlikely.",
                ),
            ],
            Vec::new(),
        );
    }

    if axis == DominantAxis::Horizontal {
        return decision(
            LayoutStrategy::HStack,
            0.82,
            "Children are horizontally distributed with aligned y positions.",
            vec![
                alternative(
                    LayoutStrategy::Absolute,
                    0.33,
                    "Absolute layout remains possible but less structured.",
                ),
                alternative(
                    LayoutStrategy::Overlay,
                    0.17,
                    "No overlap observed, so overlay is unlikely.",
                ),
            ],
            Vec::new(),
        );
    }

    decision(
        LayoutStrategy::Absolute,
        0.56,
        "Geometry is mixed and does not strongly support stack or overlay semantics.",
        vec![
            alternative(
                LayoutStrategy::VStack,
                0.49,
                "Vertical ordering is possible but weakly supported.",
            ),
            alternative(
                LayoutStrategy::HStack,
                0.47,
                "Horizontal ordering is possible but weakly supported.",
            ),
        ],
        vec![InferenceWarning {
            code: WARNING_LOW_CONFIDENCE_GEOMETRY.to_string(),
            severity: WarningSeverity::Medium,
            message: "Ambiguous child geometry reduced inference confidence.".to_string(),
            node_id: Some(node.id.clone()),
        }],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DominantAxis {
    Horizontal,
    Vertical,
    Mixed,
}

fn dominant_axis(children: &[&figma_normalizer::NormalizedNode]) -> DominantAxis {
    let min_x = children
        .iter()
        .map(|node| node.bounds.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = children
        .iter()
        .map(|node| node.bounds.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = children
        .iter()
        .map(|node| node.bounds.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = children
        .iter()
        .map(|node| node.bounds.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let span_x = max_x - min_x;
    let span_y = max_y - min_y;
    let threshold = 1.2;

    if span_y > span_x * threshold {
        DominantAxis::Vertical
    } else if span_x > span_y * threshold {
        DominantAxis::Horizontal
    } else {
        DominantAxis::Mixed
    }
}

fn has_overlaps(children: &[&figma_normalizer::NormalizedNode]) -> bool {
    for (index, first) in children.iter().enumerate() {
        for second in children.iter().skip(index + 1) {
            if intersects(first, second) {
                return true;
            }
        }
    }
    false
}

fn intersects(
    first: &figma_normalizer::NormalizedNode,
    second: &figma_normalizer::NormalizedNode,
) -> bool {
    let first_right = first.bounds.x + first.bounds.w;
    let second_right = second.bounds.x + second.bounds.w;
    let first_bottom = first.bounds.y + first.bounds.h;
    let second_bottom = second.bounds.y + second.bounds.h;

    first.bounds.x < second_right
        && second.bounds.x < first_right
        && first.bounds.y < second_bottom
        && second.bounds.y < first_bottom
}

fn decision(
    selected_strategy: LayoutStrategy,
    confidence: f32,
    rationale: &str,
    alternatives: Vec<LayoutAlternative>,
    warnings: Vec<InferenceWarning>,
) -> LayoutDecisionRecord {
    LayoutDecisionRecord {
        decision_version: LAYOUT_DECISION_VERSION.to_string(),
        selected_strategy,
        confidence,
        rationale: rationale.to_string(),
        alternatives,
        warnings,
    }
}

fn alternative(strategy: LayoutStrategy, score: f32, rationale: &str) -> LayoutAlternative {
    LayoutAlternative {
        strategy,
        score,
        rationale: rationale.to_string(),
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

    #[test]
    fn infer_layout_prefers_explicit_vertical_layout_metadata() {
        let document = normalized_document_with_layout(figma_normalizer::LayoutMode::Vertical);
        let inferred = super::infer_layout(&document);

        let root = inferred
            .decisions
            .iter()
            .find(|decision| decision.node_id == "1:1")
            .expect("root decision should exist");
        assert_eq!(root.record.selected_strategy, LayoutStrategy::VStack);
        assert!(root.record.confidence >= 0.95);
        assert!(root.record.warnings.is_empty());
    }

    #[test]
    fn infer_layout_uses_geometry_for_horizontal_stack_when_layout_is_missing() {
        let document = normalized_document_without_layout_horizontal();
        let inferred = super::infer_layout(&document);

        let root = inferred
            .decisions
            .iter()
            .find(|decision| decision.node_id == "1:1")
            .expect("root decision should exist");
        assert_eq!(root.record.selected_strategy, LayoutStrategy::HStack);
        assert!(root.record.confidence >= 0.75);
    }

    #[test]
    fn infer_layout_emits_low_confidence_warning_for_ambiguous_geometry() {
        let document = normalized_document_without_layout_ambiguous();
        let inferred = super::infer_layout(&document);

        let root = inferred
            .decisions
            .iter()
            .find(|decision| decision.node_id == "1:1")
            .expect("root decision should exist");
        assert_eq!(root.record.selected_strategy, LayoutStrategy::Absolute);
        assert!(root.record.confidence < 0.7);
        assert!(
            root.record
                .warnings
                .iter()
                .any(|warning| warning.code == super::WARNING_LOW_CONFIDENCE_GEOMETRY)
        );
    }

    #[test]
    fn warning_kind_maps_known_codes() {
        assert_eq!(
            super::warning_kind(super::WARNING_LOW_CONFIDENCE_GEOMETRY),
            super::InferenceWarningKind::LowConfidence
        );
        assert_eq!(
            super::warning_kind(super::WARNING_UNSUPPORTED_NODE_KIND),
            super::InferenceWarningKind::UnsupportedFeature
        );
        assert_eq!(
            super::warning_kind("UNKNOWN"),
            super::InferenceWarningKind::FallbackApplied
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

    fn normalized_document_with_layout(
        mode: figma_normalizer::LayoutMode,
    ) -> figma_normalizer::NormalizedDocument {
        figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    None,
                    Some(figma_normalizer::LayoutMetadata {
                        mode,
                        primary_align: figma_normalizer::Align::Start,
                        cross_align: figma_normalizer::Align::Start,
                        item_spacing: 8.0,
                        padding: figma_normalizer::Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: 0.0,
                        },
                    }),
                    vec!["2:1".to_string(), "3:1".to_string()],
                ),
                text_node("2:1", Some("1:1"), 20.0, 20.0),
                text_node("3:1", Some("1:1"), 20.0, 80.0),
            ],
        }
    }

    fn normalized_document_without_layout_horizontal() -> figma_normalizer::NormalizedDocument {
        figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    None,
                    None,
                    vec!["2:1".to_string(), "3:1".to_string()],
                ),
                text_node("2:1", Some("1:1"), 20.0, 20.0),
                text_node("3:1", Some("1:1"), 120.0, 20.0),
            ],
        }
    }

    fn normalized_document_without_layout_ambiguous() -> figma_normalizer::NormalizedDocument {
        figma_normalizer::NormalizedDocument {
            schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
            source: figma_normalizer::NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                container_node(
                    "1:1",
                    None,
                    None,
                    vec!["2:1".to_string(), "3:1".to_string()],
                ),
                text_node("2:1", Some("1:1"), 20.0, 20.0),
                text_node("3:1", Some("1:1"), 140.0, 140.0),
            ],
        }
    }

    fn container_node(
        id: &str,
        parent_id: Option<&str>,
        layout: Option<figma_normalizer::LayoutMetadata>,
        children: Vec<String>,
    ) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: parent_id.map(str::to_string),
            name: "Container".to_string(),
            kind: figma_normalizer::NodeKind::Frame,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: 300.0,
                h: 300.0,
            },
            layout,
            constraints: None,
            style: default_style(),
            component: default_component(),
            passthrough_fields: std::collections::BTreeMap::new(),
            children,
        }
    }

    fn text_node(
        id: &str,
        parent_id: Option<&str>,
        x: f32,
        y: f32,
    ) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: parent_id.map(str::to_string),
            name: "Text".to_string(),
            kind: figma_normalizer::NodeKind::Text,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x,
                y,
                w: 80.0,
                h: 30.0,
            },
            layout: None,
            constraints: None,
            style: default_style(),
            component: default_component(),
            passthrough_fields: std::collections::BTreeMap::new(),
            children: Vec::new(),
        }
    }

    fn default_style() -> figma_normalizer::NodeStyle {
        figma_normalizer::NodeStyle {
            opacity: 1.0,
            corner_radius: None,
            fills: Vec::new(),
            strokes: Vec::new(),
        }
    }

    fn default_component() -> figma_normalizer::ComponentMetadata {
        figma_normalizer::ComponentMetadata {
            component_id: None,
            component_set_id: None,
            instance_of: None,
            variant_properties: Vec::new(),
        }
    }
}

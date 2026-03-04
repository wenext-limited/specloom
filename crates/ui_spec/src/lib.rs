#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub const UI_SPEC_VERSION: &str = "1.0";
pub const UI_SPEC_GENERATOR_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpec {
    pub spec_version: String,
    pub source: UiSpecSource,
    pub root: UiNode,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<UiSpecWarning>,
}

impl Default for UiSpec {
    fn default() -> Self {
        Self {
            spec_version: UI_SPEC_VERSION.to_string(),
            source: UiSpecSource::default(),
            root: UiNode::default(),
            warnings: Vec::new(),
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

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpecSource {
    pub file_key: String,
    pub root_node_id: String,
    pub generator_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpecWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiNode {
    pub id: String,
    pub name: String,
    pub kind: UiNodeKind,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub layout: UiLayout,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub style: UiStyle,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiNode>,
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: UiNodeKind::Unknown,
            layout: UiLayout::default(),
            style: UiStyle::default(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiNodeKind {
    Container,
    Text,
    Shape,
    Image,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiLayout {
    pub strategy: UiLayoutStrategy,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub item_spacing: f32,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self {
            strategy: UiLayoutStrategy::Absolute,
            item_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutStrategy {
    VStack,
    HStack,
    Overlay,
    Absolute,
    Scroll,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiStyle {
    #[serde(default = "default_opacity")]
    #[serde(skip_serializing_if = "is_default_opacity")]
    pub opacity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            corner_radius: None,
        }
    }
}

fn default_opacity() -> f32 {
    1.0
}

fn is_default_opacity(value: &f32) -> bool {
    (*value - 1.0).abs() <= f32::EPSILON
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum UiSpecBuildError {
    #[error("missing normalized root node: {0}")]
    MissingNormalizedRootNode(String),
    #[error("missing normalized node: {0}")]
    MissingNormalizedNode(String),
}

pub fn build_ui_spec(
    normalized: &figma_normalizer::NormalizationOutput,
    inferred: &layout_infer::InferredLayoutDocument,
) -> Result<UiSpec, UiSpecBuildError> {
    let nodes_by_id = normalized
        .document
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let decisions_by_id = inferred
        .decisions
        .iter()
        .map(|decision| (decision.node_id.as_str(), &decision.record))
        .collect::<BTreeMap<_, _>>();

    let root_node_id = normalized.document.source.root_node_id.clone();
    if !nodes_by_id.contains_key(root_node_id.as_str()) {
        return Err(UiSpecBuildError::MissingNormalizedRootNode(root_node_id));
    }

    let root = build_ui_node(root_node_id.as_str(), &nodes_by_id, &decisions_by_id)?;
    let mut warnings = normalized
        .warnings
        .iter()
        .map(|warning| UiSpecWarning {
            code: warning.code.clone(),
            message: warning.message.clone(),
            node_id: warning.node_id.clone(),
        })
        .collect::<Vec<_>>();
    warnings.extend(
        inferred
            .decisions
            .iter()
            .flat_map(|decision| decision.record.warnings.iter())
            .map(|warning| UiSpecWarning {
                code: warning.code.clone(),
                message: warning.message.clone(),
                node_id: warning.node_id.clone(),
            }),
    );

    Ok(UiSpec {
        spec_version: UI_SPEC_VERSION.to_string(),
        source: UiSpecSource {
            file_key: normalized.document.source.file_key.clone(),
            root_node_id: normalized.document.source.root_node_id.clone(),
            generator_version: UI_SPEC_GENERATOR_VERSION.to_string(),
        },
        root,
        warnings,
    })
}

fn build_ui_node(
    node_id: &str,
    nodes_by_id: &BTreeMap<&str, &figma_normalizer::NormalizedNode>,
    decisions_by_id: &BTreeMap<&str, &layout_infer::LayoutDecisionRecord>,
) -> Result<UiNode, UiSpecBuildError> {
    let node = nodes_by_id
        .get(node_id)
        .copied()
        .ok_or_else(|| UiSpecBuildError::MissingNormalizedNode(node_id.to_string()))?;

    let children = node
        .children
        .iter()
        .map(|child_id| build_ui_node(child_id.as_str(), nodes_by_id, decisions_by_id))
        .collect::<Result<Vec<_>, _>>()?;

    let inferred_layout = decisions_by_id.get(node.id.as_str()).copied();
    Ok(UiNode {
        id: node.id.clone(),
        name: node.name.clone(),
        kind: map_node_kind(&node.kind),
        layout: UiLayout {
            strategy: inferred_layout
                .map(|decision| map_layout_strategy(&decision.selected_strategy))
                .unwrap_or_else(|| {
                    map_layout_mode(node.layout.as_ref().map(|layout| &layout.mode))
                }),
            item_spacing: node
                .layout
                .as_ref()
                .map(|layout| layout.item_spacing)
                .unwrap_or(0.0),
        },
        style: UiStyle {
            opacity: node.style.opacity,
            corner_radius: node.style.corner_radius,
        },
        children,
    })
}

fn map_node_kind(kind: &figma_normalizer::NodeKind) -> UiNodeKind {
    match kind {
        figma_normalizer::NodeKind::Frame
        | figma_normalizer::NodeKind::Group
        | figma_normalizer::NodeKind::Component
        | figma_normalizer::NodeKind::Instance
        | figma_normalizer::NodeKind::ComponentSet => UiNodeKind::Container,
        figma_normalizer::NodeKind::Text => UiNodeKind::Text,
        figma_normalizer::NodeKind::Rectangle
        | figma_normalizer::NodeKind::Ellipse
        | figma_normalizer::NodeKind::Vector => UiNodeKind::Shape,
        figma_normalizer::NodeKind::Unknown => UiNodeKind::Unknown,
    }
}

fn map_layout_mode(mode: Option<&figma_normalizer::LayoutMode>) -> UiLayoutStrategy {
    match mode {
        Some(figma_normalizer::LayoutMode::Vertical) => UiLayoutStrategy::VStack,
        Some(figma_normalizer::LayoutMode::Horizontal) => UiLayoutStrategy::HStack,
        Some(figma_normalizer::LayoutMode::None) | None => UiLayoutStrategy::Absolute,
    }
}

fn map_layout_strategy(strategy: &layout_infer::LayoutStrategy) -> UiLayoutStrategy {
    match strategy {
        layout_infer::LayoutStrategy::VStack => UiLayoutStrategy::VStack,
        layout_infer::LayoutStrategy::HStack => UiLayoutStrategy::HStack,
        layout_infer::LayoutStrategy::Overlay => UiLayoutStrategy::Overlay,
        layout_infer::LayoutStrategy::Absolute => UiLayoutStrategy::Absolute,
        layout_infer::LayoutStrategy::Scroll => UiLayoutStrategy::Scroll,
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

    #[test]
    fn ui_spec_tree_round_trip() {
        let spec = UiSpec {
            spec_version: "1.0".to_string(),
            source: UiSpecSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            root: UiNode {
                id: "1:1".to_string(),
                name: "Root".to_string(),
                kind: UiNodeKind::Container,
                layout: UiLayout {
                    strategy: UiLayoutStrategy::VStack,
                    item_spacing: 12.0,
                },
                style: UiStyle {
                    opacity: 1.0,
                    corner_radius: Some(8.0),
                },
                children: vec![UiNode {
                    id: "2:1".to_string(),
                    name: "Title".to_string(),
                    kind: UiNodeKind::Text,
                    layout: UiLayout::default(),
                    style: UiStyle::default(),
                    children: Vec::new(),
                }],
            },
            warnings: vec![UiSpecWarning {
                code: "LOW_CONFIDENCE_LAYOUT".to_string(),
                message: "Layout was inferred from geometry.".to_string(),
                node_id: Some("1:1".to_string()),
            }],
        };

        let json = serde_json::to_string_pretty(&spec).unwrap();
        let back: UiSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, spec);
    }

    #[test]
    fn child_order_is_preserved() {
        let spec = UiSpec {
            root: UiNode {
                children: vec![
                    UiNode {
                        id: "2:1".to_string(),
                        ..UiNode::default()
                    },
                    UiNode {
                        id: "3:1".to_string(),
                        ..UiNode::default()
                    },
                ],
                ..UiNode::default()
            },
            ..UiSpec::default()
        };

        let encoded = serde_json::to_string(&spec).unwrap();
        let decoded: UiSpec = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            decoded
                .root
                .children
                .iter()
                .map(|node| node.id.clone())
                .collect::<Vec<_>>(),
            vec!["2:1".to_string(), "3:1".to_string()]
        );
    }

    #[test]
    fn empty_fields_are_omitted_and_missing_empty_fields_deserialize() {
        let encoded_default = serde_json::to_value(UiSpec::default()).unwrap();
        let object = encoded_default
            .as_object()
            .expect("ui spec should serialize as an object");
        assert!(
            !object.contains_key("warnings"),
            "warnings should be omitted when empty"
        );
        let root = object
            .get("root")
            .and_then(serde_json::Value::as_object)
            .expect("root should serialize as object");
        assert!(
            !root.contains_key("children"),
            "children should be omitted when empty"
        );

        let decoded: UiSpec = serde_json::from_str(
            r#"{
                "spec_version": "1.0",
                "source": {
                    "file_key": "abc123",
                    "root_node_id": "1:1",
                    "generator_version": "0.1.0"
                },
                "root": {
                    "id": "1:1",
                    "name": "Root",
                    "kind": "container",
                    "layout": {
                        "strategy": "absolute",
                        "item_spacing": 0.0
                    },
                    "style": {}
                }
            }"#,
        )
        .expect("ui spec should deserialize when empty fields are omitted");

        assert!(decoded.warnings.is_empty());
        assert!(decoded.root.children.is_empty());
        assert_eq!(decoded.root.style.corner_radius, None);

        assert!(
            !root.contains_key("style"),
            "style should be omitted when all style fields are default"
        );

        let encoded_with_corner_radius = serde_json::to_value(UiSpec {
            root: UiNode {
                style: UiStyle {
                    corner_radius: Some(8.0),
                    ..UiStyle::default()
                },
                ..UiNode::default()
            },
            ..UiSpec::default()
        })
        .expect("ui spec with non-default style should serialize");
        let style = encoded_with_corner_radius
            .get("root")
            .and_then(serde_json::Value::as_object)
            .expect("root should serialize as object")
            .get("style")
            .and_then(serde_json::Value::as_object)
            .expect("style should serialize as object");
        assert!(
            !style.contains_key("opacity"),
            "opacity should be omitted when default"
        );
    }

    #[test]
    fn build_ui_spec_maps_layout_and_children_from_inputs() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", None, vec!["2:1".to_string()]),
                    text_node("2:1", Some("1:1")),
                ],
            },
            warnings: vec![figma_normalizer::NormalizationWarning {
                code: "UNSUPPORTED_NODE_FIELD".to_string(),
                message: "Unsupported field ignored.".to_string(),
                node_id: Some("1:1".to_string()),
            }],
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: vec![
                layout_infer::NodeLayoutDecision {
                    node_id: "1:1".to_string(),
                    record: layout_infer::LayoutDecisionRecord {
                        selected_strategy: layout_infer::LayoutStrategy::VStack,
                        confidence: 0.93,
                        rationale: "Auto layout metadata.".to_string(),
                        alternatives: Vec::new(),
                        warnings: vec![],
                        ..layout_infer::LayoutDecisionRecord::default()
                    },
                },
                layout_infer::NodeLayoutDecision {
                    node_id: "2:1".to_string(),
                    record: layout_infer::LayoutDecisionRecord::default(),
                },
            ],
        };

        let spec = super::build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.source.file_key, "abc123");
        assert_eq!(spec.source.root_node_id, "1:1");
        assert_eq!(spec.root.layout.strategy, UiLayoutStrategy::VStack);
        assert_eq!(spec.root.children.len(), 1);
        assert_eq!(spec.root.children[0].id, "2:1");
        assert!(!spec.warnings.is_empty());
    }

    #[test]
    fn build_ui_spec_rejects_missing_root_node() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "missing-root".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![text_node("2:1", None)],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "missing-root".to_string(),
            decisions: Vec::new(),
        };

        let err =
            super::build_ui_spec(&normalized, &inferred).expect_err("missing root should fail");
        assert!(
            err.to_string()
                .contains("missing normalized root node: missing-root")
        );
    }

    fn container_node(
        id: &str,
        parent_id: Option<&str>,
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
            layout: Some(figma_normalizer::LayoutMetadata {
                mode: figma_normalizer::LayoutMode::Vertical,
                primary_align: figma_normalizer::Align::Start,
                cross_align: figma_normalizer::Align::Stretch,
                item_spacing: 12.0,
                padding: figma_normalizer::Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
            }),
            constraints: None,
            style: figma_normalizer::NodeStyle {
                opacity: 1.0,
                corner_radius: Some(8.0),
                fills: Vec::new(),
                strokes: Vec::new(),
            },
            component: figma_normalizer::ComponentMetadata {
                component_id: None,
                component_set_id: None,
                instance_of: None,
                variant_properties: Vec::new(),
            },
            passthrough_fields: std::collections::BTreeMap::new(),
            children,
        }
    }

    fn text_node(id: &str, parent_id: Option<&str>) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: parent_id.map(str::to_string),
            name: "Text".to_string(),
            kind: figma_normalizer::NodeKind::Text,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 16.0,
                y: 16.0,
                w: 120.0,
                h: 28.0,
            },
            layout: None,
            constraints: None,
            style: figma_normalizer::NodeStyle {
                opacity: 1.0,
                corner_radius: None,
                fills: Vec::new(),
                strokes: Vec::new(),
            },
            component: figma_normalizer::ComponentMetadata {
                component_id: None,
                component_set_id: None,
                instance_of: None,
                variant_properties: Vec::new(),
            },
            passthrough_fields: std::collections::BTreeMap::new(),
            children: Vec::new(),
        }
    }
}

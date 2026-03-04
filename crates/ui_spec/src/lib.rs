#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub const UI_SPEC_VERSION: &str = "2.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UiSpec {
    Container {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "String::is_empty")]
        text: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Instance {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Text {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Image {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Shape {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
    Vector {
        id: String,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<UiSpec>,
    },
}

impl Default for UiSpec {
    fn default() -> Self {
        Self::Container {
            id: String::new(),
            name: String::new(),
            text: String::new(),
            children: Vec::new(),
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

    pub fn to_pretty_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().struct_names(true))
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Container { id, .. }
            | Self::Instance { id, .. }
            | Self::Text { id, .. }
            | Self::Image { id, .. }
            | Self::Shape { id, .. }
            | Self::Vector { id, .. } => id.as_str(),
        }
    }

    pub fn children(&self) -> &[UiSpec] {
        match self {
            Self::Container { children, .. }
            | Self::Instance { children, .. }
            | Self::Text { children, .. }
            | Self::Image { children, .. }
            | Self::Shape { children, .. }
            | Self::Vector { children, .. } => children.as_slice(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Container { .. } => NodeType::Container,
            Self::Instance { .. } => NodeType::Instance,
            Self::Text { .. } => NodeType::Text,
            Self::Image { .. } => NodeType::Image,
            Self::Shape { .. } => NodeType::Shape,
            Self::Vector { .. } => NodeType::Vector,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Container,
    Instance,
    Text,
    Image,
    Shape,
    Vector,
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
    _inferred: &layout_infer::InferredLayoutDocument,
) -> Result<UiSpec, UiSpecBuildError> {
    let nodes_by_id = normalized
        .document
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    let root_node_id = normalized.document.source.root_node_id.clone();
    if !nodes_by_id.contains_key(root_node_id.as_str()) {
        return Err(UiSpecBuildError::MissingNormalizedRootNode(root_node_id));
    }

    build_ui_spec_node(root_node_id.as_str(), &nodes_by_id)
}

fn build_ui_spec_node(
    node_id: &str,
    nodes_by_id: &BTreeMap<&str, &figma_normalizer::NormalizedNode>,
) -> Result<UiSpec, UiSpecBuildError> {
    let node = nodes_by_id
        .get(node_id)
        .copied()
        .ok_or_else(|| UiSpecBuildError::MissingNormalizedNode(node_id.to_string()))?;

    let mut children = Vec::new();
    for child_id in &node.children {
        let child = nodes_by_id
            .get(child_id.as_str())
            .copied()
            .ok_or_else(|| UiSpecBuildError::MissingNormalizedNode(child_id.clone()))?;
        if !child.visible {
            continue;
        }
        children.push(build_ui_spec_node(child_id.as_str(), nodes_by_id)?);
    }

    let node_type = map_node_type(node);
    if node_type == NodeType::Container {
        if let Some(text) = single_text_child_name(children.as_slice()) {
            return Ok(UiSpec::Container {
                id: node.id.clone(),
                name: node.name.clone(),
                text,
                children: Vec::new(),
            });
        }
    }

    if node_type == NodeType::Container
        && has_at_least_one_vector_and_remaining_shapes(children.as_slice())
    {
        return Ok(UiSpec::Vector {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
    }

    if node_type == NodeType::Instance
        && has_at_least_one_vector_and_remaining_shapes(children.as_slice())
    {
        return Ok(UiSpec::Instance {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
    }

    if matches!(node_type, NodeType::Container | NodeType::Instance)
        && (has_single_image_like_and_remaining_shapes(children.as_slice())
            || has_single_image_like_child(children.as_slice()))
    {
        return Ok(UiSpec::Image {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
    }

    if node_type == NodeType::Container && all_children_are_shape(children.as_slice()) {
        return Ok(UiSpec::Shape {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
    }

    Ok(match node_type {
        NodeType::Container => UiSpec::Container {
            id: node.id.clone(),
            name: node.name.clone(),
            text: String::new(),
            children,
        },
        NodeType::Instance => UiSpec::Instance {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::Text => UiSpec::Text {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::Image => UiSpec::Image {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::Shape => UiSpec::Shape {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::Vector => UiSpec::Vector {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
    })
}

fn single_text_child_name(children: &[UiSpec]) -> Option<String> {
    match children {
        [UiSpec::Text { name, .. }] => Some(name.clone()),
        _ => None,
    }
}

fn all_children_are_shape(children: &[UiSpec]) -> bool {
    !children.is_empty()
        && children
            .iter()
            .all(|child| child.node_type() == NodeType::Shape)
}

fn has_single_image_like_and_remaining_shapes(children: &[UiSpec]) -> bool {
    if children.len() < 2 {
        return false;
    }

    let mut image_like_count = 0usize;
    let mut shape_count = 0usize;
    for child in children {
        if is_image_like(child) {
            image_like_count += 1;
            continue;
        }
        match child.node_type() {
            NodeType::Shape => shape_count += 1,
            _ => return false,
        }
    }

    image_like_count == 1 && shape_count >= 1
}

fn has_at_least_one_vector_and_remaining_shapes(children: &[UiSpec]) -> bool {
    if children.len() < 2 {
        return false;
    }

    let mut vector_count = 0usize;
    let mut shape_count = 0usize;
    for child in children {
        match child.node_type() {
            NodeType::Vector => vector_count += 1,
            NodeType::Shape => shape_count += 1,
            _ => return false,
        }
    }

    vector_count >= 1 && shape_count >= 1
}

fn has_single_image_like_child(children: &[UiSpec]) -> bool {
    matches!(children, [child] if is_image_like(child))
}

fn is_image_like(node: &UiSpec) -> bool {
    matches!(node.node_type(), NodeType::Image | NodeType::Vector)
}

fn map_node_type(node: &figma_normalizer::NormalizedNode) -> NodeType {
    match node.kind {
        figma_normalizer::NodeKind::Frame
        | figma_normalizer::NodeKind::Group
        | figma_normalizer::NodeKind::Component
        | figma_normalizer::NodeKind::ComponentSet => NodeType::Container,
        figma_normalizer::NodeKind::Instance => NodeType::Instance,
        figma_normalizer::NodeKind::Text => NodeType::Text,
        figma_normalizer::NodeKind::Rectangle
        | figma_normalizer::NodeKind::Ellipse
        | figma_normalizer::NodeKind::Star => {
            let has_image_fill = node
                .style
                .fills
                .iter()
                .any(|fill| fill.kind == figma_normalizer::PaintKind::Image);

            if has_image_fill {
                NodeType::Image
            } else {
                NodeType::Shape
            }
        }
        figma_normalizer::NodeKind::Vector => {
            let has_image_fill = node
                .style
                .fills
                .iter()
                .any(|fill| fill.kind == figma_normalizer::PaintKind::Image);

            if has_image_fill {
                NodeType::Image
            } else {
                NodeType::Vector
            }
        }
        figma_normalizer::NodeKind::Unknown => {
            if node.children.is_empty() {
                NodeType::Vector
            } else {
                NodeType::Container
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_spec_round_trip() {
        let spec = UiSpec::Container {
            id: "1:1".to_string(),
            name: "Root".to_string(),
            text: String::new(),
            children: vec![UiSpec::Text {
                id: "1:2".to_string(),
                name: "Title".to_string(),
                children: Vec::new(),
            }],
        };

        let json = serde_json::to_string(&spec).unwrap();
        let back: UiSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }

    #[test]
    fn ron_serialization_is_stable() {
        let spec = UiSpec::Container {
            id: "1:1".to_string(),
            name: "Root".to_string(),
            text: String::new(),
            children: vec![UiSpec::Vector {
                id: "1:2".to_string(),
                name: "Icon".to_string(),
                children: Vec::new(),
            }],
        };

        let first = spec.to_pretty_ron().unwrap();
        let second = spec.to_pretty_ron().unwrap();
        assert_eq!(first, second);
        assert!(first.contains("Container("));
        assert!(!first.contains("text:"));
    }

    #[test]
    fn container_text_field_omits_when_empty_and_serializes_when_present() {
        let empty_text = UiSpec::Container {
            id: "1:1".to_string(),
            name: "Root".to_string(),
            text: String::new(),
            children: Vec::new(),
        };
        let filled_text = UiSpec::Container {
            id: "1:1".to_string(),
            name: "Root".to_string(),
            text: "Title".to_string(),
            children: Vec::new(),
        };

        let empty_ron = empty_text.to_pretty_ron().unwrap();
        let filled_ron = filled_text.to_pretty_ron().unwrap();

        assert!(!empty_ron.contains("text:"));
        assert!(filled_ron.contains("text: \"Title\""));
    }

    #[test]
    fn leaf_nodes_omit_empty_children_in_ron() {
        let leaf = UiSpec::Text {
            id: "9:9".to_string(),
            name: "Leaf".to_string(),
            children: Vec::new(),
        };

        let ron = leaf.to_pretty_ron().unwrap();
        assert!(!ron.contains("children"));
    }

    #[test]
    fn build_ui_spec_preserves_original_node_ids() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                    text_node("2:1"),
                    vector_node("3:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.id(), "1:1");
        assert_eq!(spec.node_type(), NodeType::Container);
        assert_eq!(spec.children().len(), 2);
        assert_eq!(spec.children()[0].id(), "2:1");
        assert_eq!(spec.children()[0].node_type(), NodeType::Text);
        assert_eq!(spec.children()[1].id(), "3:1");
        assert_eq!(spec.children()[1].node_type(), NodeType::Vector);
    }

    #[test]
    fn build_ui_spec_marks_image_fill_nodes_as_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                    image_node("2:1"),
                    text_node("3:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Container);
        assert_eq!(spec.children()[0].node_type(), NodeType::Image);
    }

    #[test]
    fn build_ui_spec_collapses_container_with_single_text_child_into_text_field() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![container_node("1:1", vec!["2:1".to_string()]), text_node("2:1")],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Container);
        assert!(spec.children().is_empty());
        match spec {
            UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
            _ => panic!("expected container"),
        }
    }

    #[test]
    fn build_ui_spec_omits_invisible_nodes() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node(
                        "1:1",
                        vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                    ),
                    text_node("2:1"),
                    hidden_text_node("3:1"),
                    hidden_container_node("4:1", vec!["5:1".to_string()]),
                    text_node("5:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let child_ids = spec
            .children()
            .iter()
            .map(|child| child.id().to_string())
            .collect::<Vec<_>>();

        assert!(child_ids.is_empty());
        match spec {
            UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
            _ => panic!("expected container"),
        }
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
                nodes: vec![text_node("2:1")],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "missing-root".to_string(),
            decisions: Vec::new(),
        };

        let err = build_ui_spec(&normalized, &inferred).expect_err("missing root should fail");
        assert!(
            err.to_string()
                .contains("missing normalized root node: missing-root")
        );
    }

    #[test]
    fn build_ui_spec_maps_unknown_leaf_node_kind_to_vector() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![figma_normalizer::NormalizedNode {
                    id: "1:1".to_string(),
                    parent_id: None,
                    name: "Unknown".to_string(),
                    kind: figma_normalizer::NodeKind::Unknown,
                    visible: true,
                    bounds: figma_normalizer::Bounds {
                        x: 0.0,
                        y: 0.0,
                        w: 10.0,
                        h: 10.0,
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
                }],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Vector);
    }

    #[test]
    fn build_ui_spec_maps_unknown_node_with_children_to_container() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    figma_normalizer::NormalizedNode {
                        id: "1:1".to_string(),
                        parent_id: None,
                        name: "Unknown Parent".to_string(),
                        kind: figma_normalizer::NodeKind::Unknown,
                        visible: true,
                        bounds: figma_normalizer::Bounds {
                            x: 0.0,
                            y: 0.0,
                            w: 10.0,
                            h: 10.0,
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
                        children: vec!["2:1".to_string()],
                    },
                    text_node("2:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Container);
        assert!(spec.children().is_empty());
        match spec {
            UiSpec::Container { text, .. } => assert_eq!(text, "Text"),
            _ => panic!("expected container"),
        }
    }

    #[test]
    fn build_ui_spec_maps_instance_kind_to_instance() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string()]),
                    instance_node("2:1", vec!["3:1".to_string()]),
                    text_node("3:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.children()[0].node_type(), NodeType::Instance);
        assert_eq!(spec.children()[0].children().len(), 1);
        assert_eq!(spec.children()[0].children()[0].node_type(), NodeType::Text);
    }

    #[test]
    fn build_ui_spec_maps_rectangle_kind_to_shape() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                    rectangle_node("2:1"),
                    text_node("3:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Container);
        assert_eq!(spec.children()[0].node_type(), NodeType::Shape);
    }

    #[test]
    fn build_ui_spec_collapses_container_with_all_shape_children_to_shape() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "3:1".to_string()]),
                    rectangle_node("2:1"),
                    rectangle_node("3:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Shape);
        assert!(spec.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_container_with_one_image_and_shapes_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node(
                        "1:1",
                        vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                    ),
                    image_node("2:1"),
                    rectangle_node("3:1"),
                    rectangle_node("4:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Image);
        assert!(spec.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_instance_with_one_image_and_shapes_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "6:1".to_string()]),
                    instance_node(
                        "2:1",
                        vec!["3:1".to_string(), "4:1".to_string(), "5:1".to_string()],
                    ),
                    image_node("3:1"),
                    rectangle_node("4:1"),
                    rectangle_node("5:1"),
                    text_node("6:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let collapsed = &spec.children()[0];
        assert_eq!(collapsed.node_type(), NodeType::Image);
        assert!(collapsed.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_container_with_single_image_child_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string()]),
                    image_node("2:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Image);
        assert!(spec.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_instance_with_single_image_child_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "4:1".to_string()]),
                    instance_node("2:1", vec!["3:1".to_string()]),
                    image_node("3:1"),
                    text_node("4:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let collapsed = &spec.children()[0];
        assert_eq!(collapsed.node_type(), NodeType::Image);
        assert!(collapsed.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_container_with_one_vector_and_shapes_to_vector() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node(
                        "1:1",
                        vec!["2:1".to_string(), "3:1".to_string(), "4:1".to_string()],
                    ),
                    vector_node("2:1"),
                    rectangle_node("3:1"),
                    rectangle_node("4:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Vector);
        assert!(spec.children().is_empty());
    }

    #[test]
    fn build_ui_spec_drops_children_for_instance_with_one_vector_and_shapes() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "6:1".to_string()]),
                    instance_node(
                        "2:1",
                        vec!["3:1".to_string(), "4:1".to_string(), "5:1".to_string()],
                    ),
                    vector_node("3:1"),
                    rectangle_node("4:1"),
                    rectangle_node("5:1"),
                    text_node("6:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let collapsed = &spec.children()[0];
        assert_eq!(collapsed.node_type(), NodeType::Instance);
        assert!(collapsed.children().is_empty());
    }

    #[test]
    fn build_ui_spec_drops_children_for_instance_with_multiple_vectors_and_shapes() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "7:1".to_string()]),
                    instance_node(
                        "2:1",
                        vec![
                            "3:1".to_string(),
                            "4:1".to_string(),
                            "5:1".to_string(),
                            "6:1".to_string(),
                        ],
                    ),
                    vector_node("3:1"),
                    vector_node("4:1"),
                    rectangle_node("5:1"),
                    rectangle_node("6:1"),
                    text_node("7:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let collapsed = &spec.children()[0];
        assert_eq!(collapsed.node_type(), NodeType::Instance);
        assert!(collapsed.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_container_with_single_vector_child_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string()]),
                    vector_node("2:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        assert_eq!(spec.node_type(), NodeType::Image);
        assert!(spec.children().is_empty());
    }

    #[test]
    fn build_ui_spec_collapses_instance_with_single_vector_child_to_image() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    container_node("1:1", vec!["2:1".to_string(), "4:1".to_string()]),
                    instance_node("2:1", vec!["3:1".to_string()]),
                    vector_node("3:1"),
                    text_node("4:1"),
                ],
            },
            warnings: Vec::new(),
        };
        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "abc123".to_string(),
            root_node_id: "1:1".to_string(),
            decisions: Vec::new(),
        };

        let spec = build_ui_spec(&normalized, &inferred).expect("build should succeed");
        let collapsed = &spec.children()[0];
        assert_eq!(collapsed.node_type(), NodeType::Image);
        assert!(collapsed.children().is_empty());
    }

    fn container_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
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

    fn text_node(id: &str) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
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

    fn vector_node(id: &str) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
            name: "Vector".to_string(),
            kind: figma_normalizer::NodeKind::Vector,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: 24.0,
                h: 24.0,
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

    fn rectangle_node(id: &str) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
            name: "Shape".to_string(),
            kind: figma_normalizer::NodeKind::Rectangle,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: 24.0,
                h: 24.0,
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

    fn image_node(id: &str) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
            name: "Avatar Graphic".to_string(),
            kind: figma_normalizer::NodeKind::Rectangle,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: 64.0,
                h: 64.0,
            },
            layout: None,
            constraints: None,
            style: figma_normalizer::NodeStyle {
                opacity: 1.0,
                corner_radius: None,
                fills: vec![figma_normalizer::Paint {
                    kind: figma_normalizer::PaintKind::Image,
                    color: None,
                    image_ref: Some("img-ref".to_string()),
                }],
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

    fn instance_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: None,
            name: "Button Instance".to_string(),
            kind: figma_normalizer::NodeKind::Instance,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 40.0,
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
                instance_of: Some("42:7".to_string()),
                variant_properties: Vec::new(),
            },
            passthrough_fields: std::collections::BTreeMap::new(),
            children,
        }
    }

    fn hidden_text_node(id: &str) -> figma_normalizer::NormalizedNode {
        let mut node = text_node(id);
        node.visible = false;
        node
    }

    fn hidden_container_node(id: &str, children: Vec<String>) -> figma_normalizer::NormalizedNode {
        let mut node = container_node(id, children);
        node.visible = false;
        node
    }
}

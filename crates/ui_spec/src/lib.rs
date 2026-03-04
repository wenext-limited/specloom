#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub const UI_SPEC_VERSION: &str = "2.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "Node")]
pub struct UiSpec {
    pub id: u32,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiSpec>,
}

impl Default for UiSpec {
    fn default() -> Self {
        Self {
            id: 1,
            node_type: NodeType::Container,
            name: String::new(),
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
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NodeType {
    Container,
    Text,
    Image,
    Vector,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum UiSpecBuildError {
    #[error("missing normalized root node: {0}")]
    MissingNormalizedRootNode(String),
    #[error("missing normalized node: {0}")]
    MissingNormalizedNode(String),
    #[error("unsupported normalized node kind `{kind}` at node `{node_id}`")]
    UnsupportedNormalizedNodeKind { node_id: String, kind: String },
    #[error("node id overflow while assigning deterministic ids")]
    NodeIdOverflow,
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

    let mut next_id = 1u32;
    build_ui_spec_node(root_node_id.as_str(), &nodes_by_id, &mut next_id)
}

fn build_ui_spec_node(
    node_id: &str,
    nodes_by_id: &BTreeMap<&str, &figma_normalizer::NormalizedNode>,
    next_id: &mut u32,
) -> Result<UiSpec, UiSpecBuildError> {
    let node = nodes_by_id
        .get(node_id)
        .copied()
        .ok_or_else(|| UiSpecBuildError::MissingNormalizedNode(node_id.to_string()))?;

    let assigned_id = *next_id;
    *next_id = next_id
        .checked_add(1)
        .ok_or(UiSpecBuildError::NodeIdOverflow)?;

    let children = node
        .children
        .iter()
        .map(|child_id| build_ui_spec_node(child_id.as_str(), nodes_by_id, next_id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(UiSpec {
        id: assigned_id,
        node_type: map_node_type(node)?,
        name: node.name.clone(),
        children,
    })
}

fn map_node_type(node: &figma_normalizer::NormalizedNode) -> Result<NodeType, UiSpecBuildError> {
    match node.kind {
        figma_normalizer::NodeKind::Frame
        | figma_normalizer::NodeKind::Group
        | figma_normalizer::NodeKind::Component
        | figma_normalizer::NodeKind::Instance
        | figma_normalizer::NodeKind::ComponentSet => Ok(NodeType::Container),
        figma_normalizer::NodeKind::Text => Ok(NodeType::Text),
        figma_normalizer::NodeKind::Rectangle
        | figma_normalizer::NodeKind::Ellipse
        | figma_normalizer::NodeKind::Vector => {
            let has_image_fill = node
                .style
                .fills
                .iter()
                .any(|fill| fill.kind == figma_normalizer::PaintKind::Image);
            if has_image_fill {
                Ok(NodeType::Image)
            } else {
                Ok(NodeType::Vector)
            }
        }
        figma_normalizer::NodeKind::Unknown => {
            Err(UiSpecBuildError::UnsupportedNormalizedNodeKind {
                node_id: node.id.clone(),
                kind: "unknown".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_spec_round_trip() {
        let spec = UiSpec {
            id: 1,
            node_type: NodeType::Container,
            name: "Root".to_string(),
            children: vec![UiSpec {
                id: 2,
                node_type: NodeType::Text,
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
        let spec = UiSpec {
            id: 1,
            node_type: NodeType::Container,
            name: "Root".to_string(),
            children: vec![UiSpec {
                id: 2,
                node_type: NodeType::Vector,
                name: "Icon".to_string(),
                children: Vec::new(),
            }],
        };

        let first = spec.to_pretty_ron().unwrap();
        let second = spec.to_pretty_ron().unwrap();
        assert_eq!(first, second);
        assert!(first.contains("Node("));
    }

    #[test]
    fn leaf_nodes_omit_empty_children_in_ron() {
        let leaf = UiSpec {
            id: 9,
            node_type: NodeType::Text,
            name: "Leaf".to_string(),
            children: Vec::new(),
        };

        let ron = leaf.to_pretty_ron().unwrap();
        assert!(!ron.contains("children"));
    }

    #[test]
    fn build_ui_spec_maps_tree_with_deterministic_ids() {
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
        assert_eq!(spec.id, 1);
        assert_eq!(spec.node_type, NodeType::Container);
        assert_eq!(spec.children.len(), 2);
        assert_eq!(spec.children[0].id, 2);
        assert_eq!(spec.children[0].node_type, NodeType::Text);
        assert_eq!(spec.children[1].id, 3);
        assert_eq!(spec.children[1].node_type, NodeType::Vector);
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
        assert_eq!(spec.children[0].node_type, NodeType::Image);
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
    fn build_ui_spec_rejects_unknown_node_kind() {
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

        let err = build_ui_spec(&normalized, &inferred).expect_err("build should fail");
        assert!(
            err.to_string()
                .contains("unsupported normalized node kind `unknown` at node `1:1`")
        );
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
}

use std::collections::BTreeMap;

use crate::{NodeType, UiSpec};

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
    if node_type == NodeType::Container
        && let Some(text) = single_text_child_name(children.as_slice())
    {
        return Ok(UiSpec::Container {
            id: node.id.clone(),
            name: node.name.clone(),
            text,
            children: Vec::new(),
        });
    }

    if node_type == NodeType::Container && all_children_are_vector(children.as_slice()) {
        return Ok(UiSpec::Vector {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
    }

    if node_type == NodeType::Instance && all_children_are_vector(children.as_slice()) {
        return Ok(UiSpec::Vector {
            id: node.id.clone(),
            name: node.name.clone(),
            children: Vec::new(),
        });
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

fn all_children_are_vector(children: &[UiSpec]) -> bool {
    !children.is_empty()
        && children
            .iter()
            .all(|child| child.node_type() == NodeType::Vector)
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

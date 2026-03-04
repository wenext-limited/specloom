use std::collections::BTreeMap;

use crate::{
    ChildPolicyMode, NodeType, SuggestedNodeType, TransformDecision, TransformPlan, UiSpec,
};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum UiSpecBuildError {
    #[error("missing normalized root node: {0}")]
    MissingNormalizedRootNode(String),
    #[error("missing normalized node: {0}")]
    MissingNormalizedNode(String),
    #[error("invalid transform plan: {0}")]
    InvalidTransformPlan(String),
    #[error("replacement child missing after validation for node {node_id}: {child_id}")]
    ReplacementChildMissingAfterValidation { node_id: String, child_id: String },
}

pub fn build_pre_layout_spec(
    normalized: &figma_normalizer::NormalizationOutput,
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

    build_pre_layout_node(root_node_id.as_str(), &nodes_by_id)
}

pub fn apply_transform_plan(
    pre_layout: &UiSpec,
    transform_plan: &TransformPlan,
) -> Result<UiSpec, UiSpecBuildError> {
    transform_plan
        .validate_against_pre_layout(pre_layout)
        .map_err(|err| UiSpecBuildError::InvalidTransformPlan(err.to_string()))?;

    let decisions_by_node = transform_plan
        .decisions
        .iter()
        .map(|decision| (decision.node_id.as_str(), decision))
        .collect::<BTreeMap<_, _>>();

    apply_transform_node(pre_layout, &decisions_by_node)
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

fn build_pre_layout_node(
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
        children.push(build_pre_layout_node(child_id.as_str(), nodes_by_id)?);
    }

    Ok(ui_spec_from_node_type(
        map_node_type(node),
        node.id.clone(),
        node.name.clone(),
        children,
        String::new(),
    ))
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
        NodeType::Button => UiSpec::Button {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::ScrollView => UiSpec::ScrollView {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::HStack => UiSpec::HStack {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::VStack => UiSpec::VStack {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
        NodeType::ZStack => UiSpec::ZStack {
            id: node.id.clone(),
            name: node.name.clone(),
            children,
        },
    })
}

fn apply_transform_node(
    node: &UiSpec,
    decisions_by_node: &BTreeMap<&str, &TransformDecision>,
) -> Result<UiSpec, UiSpecBuildError> {
    let transformed_children = node
        .children()
        .iter()
        .map(|child| apply_transform_node(child, decisions_by_node))
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(decision) = decisions_by_node.get(node.id()) {
        let transformed_children = match decision.child_policy.mode {
            ChildPolicyMode::Keep => transformed_children,
            ChildPolicyMode::Drop => Vec::new(),
            ChildPolicyMode::ReplaceWith => {
                let mut children_by_id = transformed_children
                    .into_iter()
                    .map(|child| (child.id().to_string(), child))
                    .collect::<BTreeMap<_, _>>();
                let mut selected = Vec::with_capacity(decision.child_policy.children.len());
                for child_id in &decision.child_policy.children {
                    let child = children_by_id.remove(child_id).ok_or_else(|| {
                        UiSpecBuildError::ReplacementChildMissingAfterValidation {
                            node_id: decision.node_id.clone(),
                            child_id: child_id.clone(),
                        }
                    })?;
                    selected.push(child);
                }
                selected
            }
        };

        return Ok(ui_spec_from_suggested_type(
            decision.suggested_type,
            node.id().to_string(),
            node_name(node).to_string(),
            transformed_children,
            container_text(node),
        ));
    }

    Ok(rebuild_node_with_children(node, transformed_children))
}

fn rebuild_node_with_children(node: &UiSpec, children: Vec<UiSpec>) -> UiSpec {
    match node {
        UiSpec::Container { id, name, text, .. } => UiSpec::Container {
            id: id.clone(),
            name: name.clone(),
            text: text.clone(),
            children,
        },
        UiSpec::Instance { id, name, .. } => UiSpec::Instance {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::Text { id, name, .. } => UiSpec::Text {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::Image { id, name, .. } => UiSpec::Image {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::Shape { id, name, .. } => UiSpec::Shape {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::Vector { id, name, .. } => UiSpec::Vector {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::Button { id, name, .. } => UiSpec::Button {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::ScrollView { id, name, .. } => UiSpec::ScrollView {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::HStack { id, name, .. } => UiSpec::HStack {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::VStack { id, name, .. } => UiSpec::VStack {
            id: id.clone(),
            name: name.clone(),
            children,
        },
        UiSpec::ZStack { id, name, .. } => UiSpec::ZStack {
            id: id.clone(),
            name: name.clone(),
            children,
        },
    }
}

fn ui_spec_from_suggested_type(
    suggested_type: SuggestedNodeType,
    id: String,
    name: String,
    children: Vec<UiSpec>,
    text: String,
) -> UiSpec {
    match suggested_type {
        SuggestedNodeType::Container => UiSpec::Container {
            id,
            name,
            text,
            children,
        },
        SuggestedNodeType::Instance => UiSpec::Instance { id, name, children },
        SuggestedNodeType::Text => UiSpec::Text { id, name, children },
        SuggestedNodeType::Image => UiSpec::Image { id, name, children },
        SuggestedNodeType::Shape => UiSpec::Shape { id, name, children },
        SuggestedNodeType::Vector => UiSpec::Vector { id, name, children },
        SuggestedNodeType::Button => UiSpec::Button { id, name, children },
        SuggestedNodeType::ScrollView => UiSpec::ScrollView { id, name, children },
        SuggestedNodeType::HStack => UiSpec::HStack { id, name, children },
        SuggestedNodeType::VStack => UiSpec::VStack { id, name, children },
        SuggestedNodeType::ZStack => UiSpec::ZStack { id, name, children },
    }
}

fn ui_spec_from_node_type(
    node_type: NodeType,
    id: String,
    name: String,
    children: Vec<UiSpec>,
    text: String,
) -> UiSpec {
    match node_type {
        NodeType::Container => UiSpec::Container {
            id,
            name,
            text,
            children,
        },
        NodeType::Instance => UiSpec::Instance { id, name, children },
        NodeType::Text => UiSpec::Text { id, name, children },
        NodeType::Image => UiSpec::Image { id, name, children },
        NodeType::Shape => UiSpec::Shape { id, name, children },
        NodeType::Vector => UiSpec::Vector { id, name, children },
        NodeType::Button => UiSpec::Button { id, name, children },
        NodeType::ScrollView => UiSpec::ScrollView { id, name, children },
        NodeType::HStack => UiSpec::HStack { id, name, children },
        NodeType::VStack => UiSpec::VStack { id, name, children },
        NodeType::ZStack => UiSpec::ZStack { id, name, children },
    }
}

fn node_name(node: &UiSpec) -> &str {
    match node {
        UiSpec::Container { name, .. }
        | UiSpec::Instance { name, .. }
        | UiSpec::Text { name, .. }
        | UiSpec::Image { name, .. }
        | UiSpec::Shape { name, .. }
        | UiSpec::Vector { name, .. }
        | UiSpec::Button { name, .. }
        | UiSpec::ScrollView { name, .. }
        | UiSpec::HStack { name, .. }
        | UiSpec::VStack { name, .. }
        | UiSpec::ZStack { name, .. } => name.as_str(),
    }
}

fn container_text(node: &UiSpec) -> String {
    match node {
        UiSpec::Container { text, .. } => text.clone(),
        _ => String::new(),
    }
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

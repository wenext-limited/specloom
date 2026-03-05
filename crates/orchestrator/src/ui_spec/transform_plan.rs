use std::collections::{BTreeMap, BTreeSet};

use super::UiSpec;

pub const TRANSFORM_PLAN_VERSION: &str = "transform_plan/1.0";

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum TransformPlanValidationError {
    #[error("unsupported transform plan version: {0}")]
    UnsupportedVersion(String),
    #[error("duplicate decision for node: {0}")]
    DuplicateDecisionNode(String),
    #[error("decision node not found: {0}")]
    DecisionNodeNotFound(String),
    #[error("replace_with requires at least one child for node: {0}")]
    ReplaceWithRequiresChildren(String),
    #[error("replacement child not found for node {node_id}: {child_id}")]
    ReplacementChildNotFound { node_id: String, child_id: String },
    #[error("unexpected child list for mode {mode} on node {node_id}")]
    UnexpectedChildrenForMode {
        node_id: String,
        mode: ChildPolicyMode,
    },
    #[error("remove_self is not allowed for root node: {0}")]
    RemoveSelfNotAllowedForRoot(String),
    #[error("replacement child removed by decision for node {node_id}: {child_id}")]
    ReplacementChildRemovedByDecision { node_id: String, child_id: String },
    #[error("duplicate repeat element id in decision for node {node_id}: {repeat_id}")]
    DuplicateRepeatElementId { node_id: String, repeat_id: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformPlan {
    pub version: String,
    #[serde(default)]
    pub decisions: Vec<TransformDecision>,
}

impl Default for TransformPlan {
    fn default() -> Self {
        Self {
            version: TRANSFORM_PLAN_VERSION.to_string(),
            decisions: Vec::new(),
        }
    }
}

impl TransformPlan {
    pub fn validate_against_pre_layout(
        &self,
        pre_layout: &UiSpec,
    ) -> Result<(), TransformPlanValidationError> {
        if self.version != TRANSFORM_PLAN_VERSION {
            return Err(TransformPlanValidationError::UnsupportedVersion(
                self.version.clone(),
            ));
        }

        let mut node_ids = BTreeSet::new();
        let mut children_by_node = BTreeMap::new();
        index_ui_spec(pre_layout, &mut node_ids, &mut children_by_node);

        let mut seen_nodes = BTreeSet::new();
        let mut decision_modes = BTreeMap::new();
        for decision in &self.decisions {
            if !seen_nodes.insert(decision.node_id.as_str()) {
                return Err(TransformPlanValidationError::DuplicateDecisionNode(
                    decision.node_id.clone(),
                ));
            }
            decision_modes.insert(decision.node_id.as_str(), decision.child_policy.mode);

            let known_children =
                children_by_node
                    .get(decision.node_id.as_str())
                    .ok_or_else(|| {
                        TransformPlanValidationError::DecisionNodeNotFound(decision.node_id.clone())
                    })?;

            if decision.child_policy.mode == ChildPolicyMode::RemoveSelf
                && decision.node_id == pre_layout.id()
            {
                return Err(TransformPlanValidationError::RemoveSelfNotAllowedForRoot(
                    decision.node_id.clone(),
                ));
            }

            match decision.child_policy.mode {
                ChildPolicyMode::Keep | ChildPolicyMode::Drop | ChildPolicyMode::RemoveSelf => {
                    if !decision.child_policy.children.is_empty() {
                        return Err(TransformPlanValidationError::UnexpectedChildrenForMode {
                            node_id: decision.node_id.clone(),
                            mode: decision.child_policy.mode,
                        });
                    }
                }
                ChildPolicyMode::ReplaceWith => {
                    if decision.child_policy.children.is_empty() {
                        return Err(TransformPlanValidationError::ReplaceWithRequiresChildren(
                            decision.node_id.clone(),
                        ));
                    }
                    for child_id in &decision.child_policy.children {
                        if !node_ids.contains(child_id.as_str())
                            || !known_children.contains(child_id.as_str())
                        {
                            return Err(TransformPlanValidationError::ReplacementChildNotFound {
                                node_id: decision.node_id.clone(),
                                child_id: child_id.clone(),
                            });
                        }
                    }
                }
            }

            if let Some(repeat_element_ids) = decision.repeat_element_ids.as_ref() {
                let mut seen_repeat_ids = BTreeSet::new();
                for repeat_id in repeat_element_ids {
                    if !seen_repeat_ids.insert(repeat_id.as_str()) {
                        return Err(TransformPlanValidationError::DuplicateRepeatElementId {
                            node_id: decision.node_id.clone(),
                            repeat_id: repeat_id.clone(),
                        });
                    }
                }
            }
        }

        for decision in &self.decisions {
            if decision.child_policy.mode != ChildPolicyMode::ReplaceWith {
                continue;
            }
            for child_id in &decision.child_policy.children {
                if decision_modes.get(child_id.as_str()) == Some(&ChildPolicyMode::RemoveSelf) {
                    return Err(
                        TransformPlanValidationError::ReplacementChildRemovedByDecision {
                            node_id: decision.node_id.clone(),
                            child_id: child_id.clone(),
                        },
                    );
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformDecision {
    pub node_id: String,
    pub suggested_type: SuggestedNodeType,
    pub child_policy: ChildPolicy,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_element_ids: Option<Vec<String>>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildPolicy {
    pub mode: ChildPolicyMode,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChildPolicyMode {
    Keep,
    Drop,
    RemoveSelf,
    ReplaceWith,
}

impl std::fmt::Display for ChildPolicyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keep => write!(f, "keep"),
            Self::Drop => write!(f, "drop"),
            Self::RemoveSelf => write!(f, "remove_self"),
            Self::ReplaceWith => write!(f, "replace_with"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SuggestedNodeType {
    #[serde(rename = "Container")]
    Container,
    #[serde(rename = "Instance")]
    Instance,
    #[serde(rename = "Text")]
    Text,
    #[serde(rename = "Image")]
    Image,
    #[serde(rename = "Shape")]
    Shape,
    #[serde(rename = "Vector")]
    Vector,
    #[serde(rename = "Button")]
    Button,
    #[serde(rename = "ScrollView")]
    ScrollView,
    #[serde(rename = "HStack")]
    HStack,
    #[serde(rename = "VStack")]
    VStack,
    #[serde(rename = "ZStack")]
    ZStack,
}

fn index_ui_spec<'a>(
    node: &'a UiSpec,
    node_ids: &mut BTreeSet<&'a str>,
    children_by_node: &mut BTreeMap<&'a str, BTreeSet<&'a str>>,
) {
    let id = node.id();
    node_ids.insert(id);
    let children = node.children();
    children_by_node.insert(id, children.iter().map(|child| child.id()).collect());

    for child in children {
        index_ui_spec(child, node_ids, children_by_node);
    }
}

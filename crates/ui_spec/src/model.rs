use std::collections::BTreeSet;

pub const UI_SPEC_VERSION: &str = "2.0";

macro_rules! define_ui_spec_enum {
    (
        containers: [$($container_variant:ident),+ $(,)?],
        leaves: [$($leaf_variant:ident),+ $(,)?],
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum UiSpec {
            $(
                $container_variant {
                    id: String,
                    name: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "String::is_empty")]
                    text: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<UiSpec>,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    repeat_element_ids: Vec<String>,
                },
            )+
            $(
                $leaf_variant {
                    id: String,
                    name: String,
                    #[serde(default)]
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<UiSpec>,
                },
            )+
        }

        impl UiSpec {
            pub fn node_type(&self) -> NodeType {
                match self {
                    $(
                        UiSpec::$container_variant { .. } => NodeType::$container_variant,
                    )+
                    $(
                        UiSpec::$leaf_variant { .. } => NodeType::$leaf_variant,
                    )+
                }
            }

            pub fn id(&self) -> &str {
                match self {
                    $(
                        UiSpec::$container_variant { id, .. } => id.as_str(),
                    )+
                    $(
                        UiSpec::$leaf_variant { id, .. } => id.as_str(),
                    )+
                }
            }

            pub fn children(&self) -> &[UiSpec] {
                match self {
                    $(
                        UiSpec::$container_variant { children, .. } => children.as_slice(),
                    )+
                    $(
                        UiSpec::$leaf_variant { children, .. } => children.as_slice(),
                    )+
                }
            }

            pub fn repeat_element_ids(&self) -> &[String] {
                match self {
                    $(
                        UiSpec::$container_variant { repeat_element_ids, .. } => repeat_element_ids.as_slice(),
                    )+
                    $(
                        UiSpec::$leaf_variant { .. } => &[],
                    )+
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum NodeType {
            $(
                $container_variant,
            )+
            $(
                $leaf_variant,
            )+
        }
    };
}

define_ui_spec_enum! {
    containers: [Container, Button],
    leaves: [Instance, Text, Image, Shape, Vector, ScrollView, HStack, VStack, ZStack],
}

impl Default for UiSpec {
    fn default() -> Self {
        Self::Container {
            id: String::new(),
            name: String::new(),
            text: String::new(),
            children: Vec::new(),
            repeat_element_ids: Vec::new(),
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

    pub fn materialize_repeats(&self) -> Result<Self, UiSpecRepeatError> {
        materialize_repeats_node(self)
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum UiSpecRepeatError {
    #[error("duplicate repeat element id for node {node_id}: {child_id}")]
    DuplicateRepeatElementId { node_id: String, child_id: String },
    #[error("repeat element id is not a direct child for node {node_id}: {child_id}")]
    RepeatElementIdNotDirectChild { node_id: String, child_id: String },
}

fn materialize_repeats_node(node: &UiSpec) -> Result<UiSpec, UiSpecRepeatError> {
    let children = node
        .children()
        .iter()
        .map(materialize_repeats_node)
        .collect::<Result<Vec<_>, _>>()?;

    validate_repeat_element_ids(node, children.as_slice())?;

    Ok(match node {
        UiSpec::Container { id, name, text, .. } => UiSpec::Container {
            id: id.clone(),
            name: name.clone(),
            text: text.clone(),
            children,
            repeat_element_ids: Vec::new(),
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
        UiSpec::Button { id, name, text, .. } => UiSpec::Button {
            id: id.clone(),
            name: name.clone(),
            text: text.clone(),
            children,
            repeat_element_ids: Vec::new(),
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
    })
}

fn validate_repeat_element_ids(
    node: &UiSpec,
    children: &[UiSpec],
) -> Result<(), UiSpecRepeatError> {
    let repeat_ids = node.repeat_element_ids();
    if repeat_ids.is_empty() {
        return Ok(());
    }

    let node_id = node.id().to_string();
    let mut seen = BTreeSet::new();
    for child_id in repeat_ids {
        if !seen.insert(child_id.as_str()) {
            return Err(UiSpecRepeatError::DuplicateRepeatElementId {
                node_id,
                child_id: child_id.clone(),
            });
        }
    }

    let direct_child_ids = children
        .iter()
        .map(|child| child.id())
        .collect::<BTreeSet<_>>();
    for child_id in repeat_ids {
        if !direct_child_ids.contains(child_id.as_str()) {
            return Err(UiSpecRepeatError::RepeatElementIdNotDirectChild {
                node_id: node.id().to_string(),
                child_id: child_id.clone(),
            });
        }
    }

    Ok(())
}

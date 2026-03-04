#![forbid(unsafe_code)]

pub const UI_SPEC_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpec {
    pub spec_version: String,
    pub source: UiSpecSource,
    pub root: UiNode,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpecSource {
    pub file_key: String,
    pub root_node_id: String,
    pub generator_version: String,
}

impl Default for UiSpecSource {
    fn default() -> Self {
        Self {
            file_key: String::new(),
            root_node_id: String::new(),
            generator_version: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpecWarning {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiNode {
    pub id: String,
    pub name: String,
    pub kind: UiNodeKind,
    pub layout: UiLayout,
    pub style: UiStyle,
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
    pub opacity: f32,
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
}

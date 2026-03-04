#![forbid(unsafe_code)]

pub const UI_BLUEPRINT_VERSION: &str = "ui_blueprint/1.0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiBlueprint {
    pub version: String,
    pub document: BlueprintDocument,
    #[serde(default)]
    pub design_tokens: BlueprintDesignTokens,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<BlueprintComponent>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub screens: Vec<BlueprintScreen>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<BlueprintAsset>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<BlueprintWarning>,
}

impl Default for UiBlueprint {
    fn default() -> Self {
        Self {
            version: UI_BLUEPRINT_VERSION.to_string(),
            document: BlueprintDocument::default(),
            design_tokens: BlueprintDesignTokens::default(),
            components: Vec::new(),
            screens: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintDocument {
    pub file_key: String,
    pub root_node_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<BlueprintViewport>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintViewport {
    pub width: f32,
    pub height: f32,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintDesignTokens {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<BlueprintColorToken>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spacing: Vec<BlueprintNumberToken>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub radius: Vec<BlueprintNumberToken>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub typography: Vec<BlueprintTypographyToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintColorToken {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintNumberToken {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintTypographyToken {
    pub name: String,
    pub font_family: String,
    pub font_weight: u16,
    pub line_height: u16,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintComponent {
    pub id: String,
    pub name: String,
    pub root: BlueprintNode,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintScreen {
    pub id: String,
    pub name: String,
    pub root: BlueprintNode,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintNode {
    pub id: String,
    pub name: String,
    pub role: BlueprintNodeRole,
    pub layout: BlueprintLayout,
    pub style: BlueprintStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BlueprintContent>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub semantics: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<BlueprintNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintNodeRole {
    Container,
    Text,
    Image,
    Shape,
    ComponentInstance,
    Unknown,
}

impl Default for BlueprintNodeRole {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintLayout {
    pub kind: BlueprintLayoutKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintLayoutKind {
    StackV,
    StackH,
    Overlay,
    Absolute,
    Scroll,
}

impl Default for BlueprintLayoutKind {
    fn default() -> Self {
        Self::Absolute
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintAsset {
    pub id: String,
    pub kind: BlueprintAssetKind,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintAssetKind {
    Image,
    Vector,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

pub fn build_ui_blueprint(spec: &ui_spec::UiSpec) -> UiBlueprint {
    UiBlueprint {
        version: UI_BLUEPRINT_VERSION.to_string(),
        document: BlueprintDocument {
            file_key: spec.source.file_key.clone(),
            root_node_id: spec.source.root_node_id.clone(),
            name: spec.root.name.clone(),
            viewport: None,
        },
        design_tokens: BlueprintDesignTokens::default(),
        components: Vec::new(),
        screens: vec![BlueprintScreen {
            id: format!("screen/{}", spec.source.root_node_id),
            name: spec.root.name.clone(),
            root: map_ui_node(&spec.root),
        }],
        assets: Vec::new(),
        warnings: spec.warnings.iter().map(BlueprintWarning::from).collect(),
    }
}

fn map_ui_node(node: &ui_spec::UiNode) -> BlueprintNode {
    BlueprintNode {
        id: node.id.clone(),
        name: node.name.clone(),
        role: map_ui_node_role(&node.kind),
        layout: BlueprintLayout {
            kind: map_ui_layout_kind(&node.layout.strategy),
            gap: (node.layout.item_spacing > 0.0).then_some(node.layout.item_spacing),
        },
        style: BlueprintStyle {
            opacity: ((node.style.opacity - 1.0).abs() > 0.000_01).then_some(node.style.opacity),
            corner_radius: node.style.corner_radius,
        },
        content: map_ui_content(node),
        semantics: Vec::new(),
        children: node.children.iter().map(map_ui_node).collect(),
    }
}

fn map_ui_node_role(kind: &ui_spec::UiNodeKind) -> BlueprintNodeRole {
    match kind {
        ui_spec::UiNodeKind::Container => BlueprintNodeRole::Container,
        ui_spec::UiNodeKind::Text => BlueprintNodeRole::Text,
        ui_spec::UiNodeKind::Image => BlueprintNodeRole::Image,
        ui_spec::UiNodeKind::Shape => BlueprintNodeRole::Shape,
        ui_spec::UiNodeKind::Unknown => BlueprintNodeRole::Unknown,
    }
}

fn map_ui_layout_kind(strategy: &ui_spec::UiLayoutStrategy) -> BlueprintLayoutKind {
    match strategy {
        ui_spec::UiLayoutStrategy::VStack => BlueprintLayoutKind::StackV,
        ui_spec::UiLayoutStrategy::HStack => BlueprintLayoutKind::StackH,
        ui_spec::UiLayoutStrategy::Overlay => BlueprintLayoutKind::Overlay,
        ui_spec::UiLayoutStrategy::Absolute => BlueprintLayoutKind::Absolute,
        ui_spec::UiLayoutStrategy::Scroll => BlueprintLayoutKind::Scroll,
    }
}

fn map_ui_content(node: &ui_spec::UiNode) -> Option<BlueprintContent> {
    match node.kind {
        ui_spec::UiNodeKind::Text => Some(BlueprintContent {
            text: Some(node.name.clone()),
            asset_id: None,
        }),
        ui_spec::UiNodeKind::Image => Some(BlueprintContent {
            text: None,
            asset_id: Some(node.name.clone()),
        }),
        _ => None,
    }
}

impl From<&ui_spec::UiSpecWarning> for BlueprintWarning {
    fn from(value: &ui_spec::UiSpecWarning) -> Self {
        Self {
            code: value.code.clone(),
            message: value.message.clone(),
            node_id: value.node_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_blueprint_round_trip() {
        let blueprint = UiBlueprint {
            version: UI_BLUEPRINT_VERSION.to_string(),
            document: BlueprintDocument {
                file_key: "abc123".to_string(),
                root_node_id: "123:456".to_string(),
                name: "Login".to_string(),
                viewport: Some(BlueprintViewport {
                    width: 390.0,
                    height: 844.0,
                }),
            },
            screens: vec![BlueprintScreen {
                id: "screen/login".to_string(),
                name: "Login".to_string(),
                root: BlueprintNode {
                    id: "node/root".to_string(),
                    name: "Root".to_string(),
                    role: BlueprintNodeRole::Container,
                    layout: BlueprintLayout {
                        kind: BlueprintLayoutKind::StackV,
                        gap: Some(16.0),
                    },
                    style: BlueprintStyle::default(),
                    content: None,
                    semantics: Vec::new(),
                    children: Vec::new(),
                },
            }],
            warnings: vec![BlueprintWarning {
                code: "LOW_CONFIDENCE_LAYOUT".to_string(),
                message: "layout strategy inferred with low confidence".to_string(),
                node_id: Some("123:789".to_string()),
            }],
            ..UiBlueprint::default()
        };

        let yaml = serde_yaml::to_string(&blueprint).unwrap();
        let back: UiBlueprint = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(blueprint, back);
    }

    #[test]
    fn default_blueprint_uses_current_schema_version() {
        let blueprint = UiBlueprint::default();
        assert_eq!(blueprint.version, UI_BLUEPRINT_VERSION);
    }

    #[test]
    fn build_blueprint_from_ui_spec_maps_root_layout_and_warnings() {
        let spec = ui_spec::UiSpec {
            source: ui_spec::UiSpecSource {
                file_key: "abc123".to_string(),
                root_node_id: "123:456".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            root: ui_spec::UiNode {
                id: "123:456".to_string(),
                name: "Login".to_string(),
                kind: ui_spec::UiNodeKind::Container,
                layout: ui_spec::UiLayout {
                    strategy: ui_spec::UiLayoutStrategy::VStack,
                    item_spacing: 16.0,
                },
                style: ui_spec::UiStyle {
                    opacity: 0.8,
                    corner_radius: Some(12.0),
                },
                children: vec![ui_spec::UiNode {
                    id: "123:457".to_string(),
                    name: "Continue".to_string(),
                    kind: ui_spec::UiNodeKind::Text,
                    layout: ui_spec::UiLayout {
                        strategy: ui_spec::UiLayoutStrategy::Absolute,
                        item_spacing: 0.0,
                    },
                    style: ui_spec::UiStyle::default(),
                    children: Vec::new(),
                }],
            },
            warnings: vec![ui_spec::UiSpecWarning {
                code: "LOW_CONFIDENCE_LAYOUT".to_string(),
                message: "layout inferred with low confidence".to_string(),
                node_id: Some("123:456".to_string()),
            }],
            ..ui_spec::UiSpec::default()
        };

        let blueprint = build_ui_blueprint(&spec);
        assert_eq!(blueprint.version, UI_BLUEPRINT_VERSION);
        assert_eq!(blueprint.document.file_key, "abc123");
        assert_eq!(blueprint.document.root_node_id, "123:456");
        assert_eq!(blueprint.document.name, "Login");
        assert_eq!(blueprint.screens.len(), 1);
        assert_eq!(blueprint.screens[0].root.layout.kind, BlueprintLayoutKind::StackV);
        assert_eq!(blueprint.screens[0].root.layout.gap, Some(16.0));
        assert_eq!(blueprint.screens[0].root.children.len(), 1);
        assert_eq!(
            blueprint.screens[0].root.children[0].role,
            BlueprintNodeRole::Text
        );
        assert_eq!(
            blueprint.screens[0].root.children[0].content,
            Some(BlueprintContent {
                text: Some("Continue".to_string()),
                asset_id: None,
            })
        );
        assert_eq!(
            blueprint.warnings,
            vec![BlueprintWarning {
                code: "LOW_CONFIDENCE_LAYOUT".to_string(),
                message: "layout inferred with low confidence".to_string(),
                node_id: Some("123:456".to_string()),
            }]
        );
    }
}

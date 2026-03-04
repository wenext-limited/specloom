#![forbid(unsafe_code)]

pub const SWIFTUI_AST_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwiftUiAst {
    pub ast_version: String,
    pub view_name: String,
    pub root: SwiftUiNode,
}

impl Default for SwiftUiAst {
    fn default() -> Self {
        Self {
            ast_version: SWIFTUI_AST_VERSION.to_string(),
            view_name: "GeneratedView".to_string(),
            root: SwiftUiNode::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwiftUiNode {
    pub kind: SwiftUiNodeKind,
    pub modifiers: Vec<SwiftUiModifier>,
    pub children: Vec<SwiftUiNode>,
}

impl Default for SwiftUiNode {
    fn default() -> Self {
        Self {
            kind: SwiftUiNodeKind::VStack { spacing: None },
            modifiers: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SwiftUiNodeKind {
    VStack { spacing: Option<f32> },
    HStack { spacing: Option<f32> },
    ZStack,
    Text { content: String },
    Image { asset_name: String },
    Spacer,
    Rectangle,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SwiftUiModifier {
    PaddingAll(f32),
    Opacity(f32),
    CornerRadius(f32),
    ForegroundColor {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    },
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SwiftUiAstBuildError {
    #[error("spec root node is required")]
    MissingRootNode,
}

pub fn build_ast_from_ui_spec(spec: &ui_spec::UiSpec) -> Result<SwiftUiAst, SwiftUiAstBuildError> {
    if spec.root.id.is_empty() {
        return Err(SwiftUiAstBuildError::MissingRootNode);
    }

    Ok(SwiftUiAst {
        ast_version: SWIFTUI_AST_VERSION.to_string(),
        view_name: swift_view_name(spec.root.name.as_str()),
        root: map_ui_node(&spec.root),
    })
}

fn swift_view_name(root_name: &str) -> String {
    let sanitized = root_name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    if sanitized.is_empty() {
        "GeneratedView".to_string()
    } else {
        format!("{sanitized}View")
    }
}

fn map_ui_node(node: &ui_spec::UiNode) -> SwiftUiNode {
    let children = node.children.iter().map(map_ui_node).collect::<Vec<_>>();

    let mut modifiers = Vec::new();
    if (node.style.opacity - 1.0).abs() > 0.000_01 {
        modifiers.push(SwiftUiModifier::Opacity(node.style.opacity));
    }
    if let Some(corner_radius) = node.style.corner_radius {
        modifiers.push(SwiftUiModifier::CornerRadius(corner_radius));
    }

    SwiftUiNode {
        kind: map_ui_kind(node),
        modifiers,
        children,
    }
}

fn map_ui_kind(node: &ui_spec::UiNode) -> SwiftUiNodeKind {
    match node.kind {
        ui_spec::UiNodeKind::Container => match node.layout.strategy {
            ui_spec::UiLayoutStrategy::VStack => SwiftUiNodeKind::VStack {
                spacing: if node.layout.item_spacing > 0.0 {
                    Some(node.layout.item_spacing)
                } else {
                    None
                },
            },
            ui_spec::UiLayoutStrategy::HStack => SwiftUiNodeKind::HStack {
                spacing: if node.layout.item_spacing > 0.0 {
                    Some(node.layout.item_spacing)
                } else {
                    None
                },
            },
            ui_spec::UiLayoutStrategy::Overlay | ui_spec::UiLayoutStrategy::Absolute => {
                SwiftUiNodeKind::ZStack
            }
            ui_spec::UiLayoutStrategy::Scroll => SwiftUiNodeKind::VStack {
                spacing: if node.layout.item_spacing > 0.0 {
                    Some(node.layout.item_spacing)
                } else {
                    None
                },
            },
        },
        ui_spec::UiNodeKind::Text => SwiftUiNodeKind::Text {
            content: node.name.clone(),
        },
        ui_spec::UiNodeKind::Image => SwiftUiNodeKind::Image {
            asset_name: node.name.clone(),
        },
        ui_spec::UiNodeKind::Shape | ui_spec::UiNodeKind::Unknown => SwiftUiNodeKind::Rectangle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swiftui_ast_round_trip() {
        let ast = SwiftUiAst::default();
        let json = serde_json::to_string(&ast).unwrap();
        let back: SwiftUiAst = serde_json::from_str(&json).unwrap();
        assert_eq!(ast, back);
    }

    #[test]
    fn ast_supports_core_view_and_modifier_primitives() {
        let ast = SwiftUiAst {
            ast_version: SWIFTUI_AST_VERSION.to_string(),
            view_name: "GeneratedView".to_string(),
            root: SwiftUiNode {
                kind: SwiftUiNodeKind::VStack {
                    spacing: Some(12.0),
                },
                modifiers: vec![
                    SwiftUiModifier::PaddingAll(16.0),
                    SwiftUiModifier::Opacity(1.0),
                ],
                children: vec![
                    SwiftUiNode {
                        kind: SwiftUiNodeKind::Text {
                            content: "Hello".to_string(),
                        },
                        modifiers: vec![SwiftUiModifier::ForegroundColor {
                            red: 0.2,
                            green: 0.2,
                            blue: 0.2,
                            alpha: 1.0,
                        }],
                        children: Vec::new(),
                    },
                    SwiftUiNode {
                        kind: SwiftUiNodeKind::Image {
                            asset_name: "hero".to_string(),
                        },
                        modifiers: vec![SwiftUiModifier::CornerRadius(8.0)],
                        children: Vec::new(),
                    },
                ],
            },
        };

        let encoded = serde_json::to_string_pretty(&ast).unwrap();
        let decoded: SwiftUiAst = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, ast);
    }

    #[test]
    fn child_order_is_stable_for_container_nodes() {
        let ast = SwiftUiAst {
            view_name: "GeneratedView".to_string(),
            root: SwiftUiNode {
                kind: SwiftUiNodeKind::HStack {
                    spacing: Some(10.0),
                },
                modifiers: Vec::new(),
                children: vec![
                    SwiftUiNode {
                        kind: SwiftUiNodeKind::Text {
                            content: "Left".to_string(),
                        },
                        modifiers: Vec::new(),
                        children: Vec::new(),
                    },
                    SwiftUiNode {
                        kind: SwiftUiNodeKind::Text {
                            content: "Right".to_string(),
                        },
                        modifiers: Vec::new(),
                        children: Vec::new(),
                    },
                ],
            },
            ..SwiftUiAst::default()
        };

        let encoded = serde_json::to_string(&ast).unwrap();
        let decoded: SwiftUiAst = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            decoded
                .root
                .children
                .iter()
                .map(|node| match &node.kind {
                    SwiftUiNodeKind::Text { content } => content.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>(),
            vec!["Left".to_string(), "Right".to_string()]
        );
    }

    #[test]
    fn build_ast_from_ui_spec_maps_tree_and_layout() {
        let spec = ui_spec::UiSpec {
            source: ui_spec::UiSpecSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            root: ui_spec::UiNode {
                id: "1:1".to_string(),
                name: "Root View".to_string(),
                kind: ui_spec::UiNodeKind::Container,
                layout: ui_spec::UiLayout {
                    strategy: ui_spec::UiLayoutStrategy::VStack,
                    item_spacing: 12.0,
                },
                style: ui_spec::UiStyle {
                    opacity: 0.9,
                    corner_radius: Some(8.0),
                },
                children: vec![ui_spec::UiNode {
                    id: "2:1".to_string(),
                    name: "Title".to_string(),
                    kind: ui_spec::UiNodeKind::Text,
                    layout: ui_spec::UiLayout::default(),
                    style: ui_spec::UiStyle::default(),
                    children: Vec::new(),
                }],
            },
            warnings: Vec::new(),
            ..ui_spec::UiSpec::default()
        };

        let ast = super::build_ast_from_ui_spec(&spec).expect("mapping should succeed");
        assert_eq!(ast.view_name, "RootViewView");
        assert_eq!(
            ast.root.kind,
            SwiftUiNodeKind::VStack {
                spacing: Some(12.0)
            }
        );
        assert!(ast.root.modifiers.contains(&SwiftUiModifier::Opacity(0.9)));
        assert!(
            ast.root
                .modifiers
                .contains(&SwiftUiModifier::CornerRadius(8.0))
        );
        assert_eq!(ast.root.children.len(), 1);
        assert_eq!(
            ast.root.children[0].kind,
            SwiftUiNodeKind::Text {
                content: "Title".to_string()
            }
        );
    }
}

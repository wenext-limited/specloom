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
                kind: SwiftUiNodeKind::VStack { spacing: Some(12.0) },
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
                kind: SwiftUiNodeKind::HStack { spacing: Some(10.0) },
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
}

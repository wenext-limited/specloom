#![forbid(unsafe_code)]

pub fn render_swift_file(ast: &swiftui_ast::SwiftUiAst) -> String {
    let mut out = String::new();
    out.push_str("import SwiftUI\n\n");
    out.push_str(&format!("struct {}: View {{\n", ast.view_name));
    out.push_str("    var body: some View {\n");
    render_node(&ast.root, 2, &mut out);
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

fn render_node(node: &swiftui_ast::SwiftUiNode, indent_level: usize, out: &mut String) {
    let indent = indent(indent_level);
    match &node.kind {
        swiftui_ast::SwiftUiNodeKind::VStack { spacing } => {
            out.push_str(&indent);
            out.push_str("VStack");
            if let Some(spacing) = spacing {
                out.push_str(&format!("(spacing: {})", format_number(*spacing)));
            }
            out.push_str(" {\n");
            for child in &node.children {
                render_node(child, indent_level + 1, out);
            }
            out.push_str(&indent);
            out.push_str("}\n");
            render_modifiers(node, indent_level, out);
        }
        swiftui_ast::SwiftUiNodeKind::HStack { spacing } => {
            out.push_str(&indent);
            out.push_str("HStack");
            if let Some(spacing) = spacing {
                out.push_str(&format!("(spacing: {})", format_number(*spacing)));
            }
            out.push_str(" {\n");
            for child in &node.children {
                render_node(child, indent_level + 1, out);
            }
            out.push_str(&indent);
            out.push_str("}\n");
            render_modifiers(node, indent_level, out);
        }
        swiftui_ast::SwiftUiNodeKind::ZStack => {
            out.push_str(&indent);
            out.push_str("ZStack {\n");
            for child in &node.children {
                render_node(child, indent_level + 1, out);
            }
            out.push_str(&indent);
            out.push_str("}\n");
            render_modifiers(node, indent_level, out);
        }
        swiftui_ast::SwiftUiNodeKind::Text { content } => {
            out.push_str(&indent);
            out.push_str(&format!("Text(\"{}\")\n", escape_swift_string(content)));
            render_modifiers(node, indent_level + 1, out);
        }
        swiftui_ast::SwiftUiNodeKind::Image { asset_name } => {
            out.push_str(&indent);
            out.push_str(&format!("Image(\"{}\")\n", escape_swift_string(asset_name)));
            render_modifiers(node, indent_level + 1, out);
        }
        swiftui_ast::SwiftUiNodeKind::Spacer => {
            out.push_str(&indent);
            out.push_str("Spacer()\n");
            render_modifiers(node, indent_level + 1, out);
        }
        swiftui_ast::SwiftUiNodeKind::Rectangle => {
            out.push_str(&indent);
            out.push_str("Rectangle()\n");
            render_modifiers(node, indent_level + 1, out);
        }
    }
}

fn render_modifiers(node: &swiftui_ast::SwiftUiNode, indent_level: usize, out: &mut String) {
    for modifier in &node.modifiers {
        let indent = indent(indent_level);
        out.push_str(&indent);
        match modifier {
            swiftui_ast::SwiftUiModifier::PaddingAll(value) => {
                out.push_str(&format!(".padding({})\n", format_number(*value)));
            }
            swiftui_ast::SwiftUiModifier::Opacity(value) => {
                out.push_str(&format!(".opacity({})\n", format_number(*value)));
            }
            swiftui_ast::SwiftUiModifier::CornerRadius(value) => {
                out.push_str(&format!(".cornerRadius({})\n", format_number(*value)));
            }
            swiftui_ast::SwiftUiModifier::ForegroundColor {
                red,
                green,
                blue,
                alpha,
            } => {
                out.push_str(&format!(
                    ".foregroundColor(Color(red: {}, green: {}, blue: {}, opacity: {}))\n",
                    format_color_component(*red),
                    format_color_component(*green),
                    format_color_component(*blue),
                    format_color_component(*alpha)
                ));
            }
        }
    }
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}

fn format_number(value: f32) -> String {
    if (value - value.round()).abs() < 0.000_01 {
        (value.round() as i64).to_string()
    } else {
        trim_trailing_zeros(value)
    }
}

fn format_color_component(value: f32) -> String {
    if (value - value.round()).abs() < 0.000_01 {
        format!("{value:.1}")
    } else {
        trim_trailing_zeros(value)
    }
}

fn trim_trailing_zeros(value: f32) -> String {
    let mut rendered = format!("{value:.3}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    rendered
}

fn escape_swift_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use swiftui_ast::{SwiftUiAst, SwiftUiModifier, SwiftUiNode, SwiftUiNodeKind};

    #[test]
    fn render_swift_file_emits_stable_swift_source() {
        let ast = SwiftUiAst {
            view_name: "GeneratedView".to_string(),
            root: SwiftUiNode {
                kind: SwiftUiNodeKind::VStack {
                    spacing: Some(12.0),
                },
                modifiers: vec![SwiftUiModifier::PaddingAll(16.0)],
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
            ..SwiftUiAst::default()
        };

        let source = super::render_swift_file(&ast);
        assert_eq!(
            source,
            "import SwiftUI\n\nstruct GeneratedView: View {\n    var body: some View {\n        VStack(spacing: 12) {\n            Text(\"Hello\")\n                .foregroundColor(Color(red: 0.2, green: 0.2, blue: 0.2, opacity: 1.0))\n            Image(\"hero\")\n                .cornerRadius(8)\n        }\n        .padding(16)\n    }\n}\n"
        );
    }

    #[test]
    fn renderer_is_deterministic_for_identical_ast() {
        let ast = SwiftUiAst {
            view_name: "RepeatView".to_string(),
            root: SwiftUiNode {
                kind: SwiftUiNodeKind::HStack {
                    spacing: Some(10.0),
                },
                modifiers: vec![SwiftUiModifier::Opacity(1.0)],
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

        let first = super::render_swift_file(&ast);
        let second = super::render_swift_file(&ast);
        assert_eq!(first, second);
    }
}

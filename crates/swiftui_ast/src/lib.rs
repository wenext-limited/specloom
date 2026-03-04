#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SwiftUiAst {
    pub ast_version: String,
}

impl Default for SwiftUiAst {
    fn default() -> Self {
        Self {
            ast_version: "1.0".to_string(),
        }
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
}

#![forbid(unsafe_code)]

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

pub fn pipeline_stage_names() -> Vec<&'static str> {
    vec![
        "fetch",
        "normalize",
        "infer-layout",
        "build-spec",
        "gen-swiftui",
        "export-assets",
        "report",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_are_reported_in_order() {
        let stages = pipeline_stage_names();
        assert_eq!(
            stages,
            vec![
                "fetch",
                "normalize",
                "infer-layout",
                "build-spec",
                "gen-swiftui",
                "export-assets",
                "report",
            ]
        );
    }

    #[test]
    fn unsupported_feature_is_classified() {
        let err = PipelineError::UnsupportedFeature("mask".to_string());
        assert!(err.to_string().contains("unsupported"));
    }
}

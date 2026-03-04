#![forbid(unsafe_code)]

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStageDefinition {
    pub name: &'static str,
    pub output_dir: &'static str,
}

const PIPELINE_STAGES: [PipelineStageDefinition; 7] = [
    PipelineStageDefinition {
        name: "fetch",
        output_dir: "output/raw",
    },
    PipelineStageDefinition {
        name: "normalize",
        output_dir: "output/normalized",
    },
    PipelineStageDefinition {
        name: "infer-layout",
        output_dir: "output/inferred",
    },
    PipelineStageDefinition {
        name: "build-spec",
        output_dir: "output/specs",
    },
    PipelineStageDefinition {
        name: "gen-swiftui",
        output_dir: "output/swift",
    },
    PipelineStageDefinition {
        name: "export-assets",
        output_dir: "output/assets",
    },
    PipelineStageDefinition {
        name: "report",
        output_dir: "output/reports",
    },
];

pub fn pipeline_stage_names() -> Vec<&'static str> {
    PIPELINE_STAGES.iter().map(|stage| stage.name).collect()
}

pub fn pipeline_stage_output_dirs() -> Vec<(&'static str, &'static str)> {
    PIPELINE_STAGES
        .iter()
        .map(|stage| (stage.name, stage.output_dir))
        .collect()
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

    #[test]
    fn stages_map_to_output_directories() {
        assert_eq!(
            pipeline_stage_output_dirs(),
            vec![
                ("fetch", "output/raw"),
                ("normalize", "output/normalized"),
                ("infer-layout", "output/inferred"),
                ("build-spec", "output/specs"),
                ("gen-swiftui", "output/swift"),
                ("export-assets", "output/assets"),
                ("report", "output/reports"),
            ]
        );
    }
}

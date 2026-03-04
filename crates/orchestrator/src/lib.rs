#![forbid(unsafe_code)]

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PipelineError {
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
    #[error("unknown stage: {0}")]
    UnknownStage(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStageDefinition {
    pub name: &'static str,
    pub output_dir: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageExecutionResult {
    pub stage_name: &'static str,
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

pub fn run_stage(stage_name: &str) -> Result<StageExecutionResult, PipelineError> {
    let stage = PIPELINE_STAGES
        .iter()
        .copied()
        .find(|candidate| candidate.name == stage_name)
        .ok_or_else(|| PipelineError::UnknownStage(stage_name.to_string()))?;

    Ok(StageExecutionResult {
        stage_name: stage.name,
        output_dir: stage.output_dir,
    })
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

    #[test]
    fn run_stage_returns_execution_result_for_known_stage() {
        let result = run_stage("normalize").expect("known stage should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "normalize",
                output_dir: "output/normalized",
            }
        );
    }

    #[test]
    fn run_stage_returns_error_for_unknown_stage() {
        let err = run_stage("not-a-stage").expect_err("unknown stage should fail");
        assert_eq!(err, PipelineError::UnknownStage("not-a-stage".to_string()));
    }
}

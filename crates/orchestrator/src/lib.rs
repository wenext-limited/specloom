#![forbid(unsafe_code)]

use std::path::Path;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PipelineError {
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
    #[error("unknown stage: {0}")]
    UnknownStage(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("fetch client error: {0}")]
    FetchClient(String),
    #[error("missing input artifact: {0}")]
    MissingInputArtifact(String),
    #[error("normalizer error: {0}")]
    Normalizer(String),
    #[error("ui spec build error: {0}")]
    UiSpecBuild(String),
    #[error("swiftui ast build error: {0}")]
    SwiftUiAstBuild(String),
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
    pub artifact_path: Option<String>,
}

const FETCH_ARTIFACT_RELATIVE_PATH: &str = "output/raw/fetch_snapshot.json";
const NORMALIZED_ARTIFACT_RELATIVE_PATH: &str = "output/normalized/normalized_document.json";
const INFERRED_ARTIFACT_RELATIVE_PATH: &str = "output/inferred/layout_inference.json";
const SPEC_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_spec.json";
const SWIFT_ARTIFACT_OUTPUT_DIR: &str = "output/swift";
const FETCH_FIXTURE_FILE_KEY: &str = "fixture-file-key";
const FETCH_FIXTURE_NODE_ID: &str = "0:1";
const FETCH_FIXTURE_JSON: &str = r#"{
  "document": {
    "id": "0:1",
    "name": "Fixture Root",
    "type": "FRAME",
    "children": []
  }
}"#;

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
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    run_stage_in_workspace(stage_name, workspace_root.as_path())
}

pub fn run_stage_in_workspace(
    stage_name: &str,
    workspace_root: &Path,
) -> Result<StageExecutionResult, PipelineError> {
    let stage = PIPELINE_STAGES
        .iter()
        .copied()
        .find(|candidate| candidate.name == stage_name)
        .ok_or_else(|| PipelineError::UnknownStage(stage_name.to_string()))?;

    let artifact_path = match stage.name {
        "fetch" => Some(run_fetch_stage(workspace_root)?),
        "normalize" => Some(run_normalize_stage(workspace_root)?),
        "infer-layout" => Some(run_infer_layout_stage(workspace_root)?),
        "build-spec" => Some(run_build_spec_stage(workspace_root)?),
        "gen-swiftui" => Some(run_gen_swiftui_stage(workspace_root)?),
        _ => None,
    };

    Ok(StageExecutionResult {
        stage_name: stage.name,
        output_dir: stage.output_dir,
        artifact_path,
    })
}

fn run_fetch_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let request = figma_client::FetchNodesRequest::new(
        FETCH_FIXTURE_FILE_KEY.to_string(),
        FETCH_FIXTURE_NODE_ID.to_string(),
    )
    .map_err(fetch_error)?;

    let snapshot = figma_client::fetch_snapshot_from_fixture(&request, FETCH_FIXTURE_JSON)
        .map_err(fetch_error)?;

    let artifact_path = workspace_root.join(FETCH_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = artifact_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }

    let encoded = serde_json::to_string_pretty(&snapshot).map_err(serialization_error)?;
    std::fs::write(&artifact_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(FETCH_ARTIFACT_RELATIVE_PATH.to_string())
}

fn run_normalize_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let raw_artifact_path = workspace_root.join(FETCH_ARTIFACT_RELATIVE_PATH);
    if !raw_artifact_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            FETCH_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let raw_artifact = std::fs::read_to_string(&raw_artifact_path).map_err(io_error)?;
    let raw_snapshot: figma_client::RawFigmaSnapshot =
        serde_json::from_str(&raw_artifact).map_err(serialization_error)?;
    let normalized =
        figma_normalizer::normalize_snapshot(&raw_snapshot).map_err(normalizer_error)?;

    let normalized_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = normalized_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = serde_json::to_string_pretty(&normalized).map_err(serialization_error)?;
    std::fs::write(&normalized_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(NORMALIZED_ARTIFACT_RELATIVE_PATH.to_string())
}

fn run_infer_layout_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    if !normalized_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            NORMALIZED_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let normalized_artifact = std::fs::read_to_string(&normalized_path).map_err(io_error)?;
    let normalized: figma_normalizer::NormalizationOutput =
        serde_json::from_str(&normalized_artifact).map_err(serialization_error)?;
    let inferred = layout_infer::infer_layout(&normalized.document);

    let inferred_path = workspace_root.join(INFERRED_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = inferred_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = serde_json::to_string_pretty(&inferred).map_err(serialization_error)?;
    std::fs::write(&inferred_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(INFERRED_ARTIFACT_RELATIVE_PATH.to_string())
}

fn run_build_spec_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    if !normalized_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            NORMALIZED_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }
    let inferred_path = workspace_root.join(INFERRED_ARTIFACT_RELATIVE_PATH);
    if !inferred_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            INFERRED_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let normalized_artifact = std::fs::read_to_string(&normalized_path).map_err(io_error)?;
    let normalized: figma_normalizer::NormalizationOutput =
        serde_json::from_str(&normalized_artifact).map_err(serialization_error)?;

    let inferred_artifact = std::fs::read_to_string(&inferred_path).map_err(io_error)?;
    let inferred: layout_infer::InferredLayoutDocument =
        serde_json::from_str(&inferred_artifact).map_err(serialization_error)?;

    let spec = ui_spec::build_ui_spec(&normalized, &inferred).map_err(ui_spec_build_error)?;

    let spec_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = spec_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = serde_json::to_string_pretty(&spec).map_err(serialization_error)?;
    std::fs::write(&spec_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(SPEC_ARTIFACT_RELATIVE_PATH.to_string())
}

fn run_gen_swiftui_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let spec_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    if !spec_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            SPEC_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let spec_artifact = std::fs::read_to_string(&spec_path).map_err(io_error)?;
    let spec: ui_spec::UiSpec =
        serde_json::from_str(&spec_artifact).map_err(serialization_error)?;
    let ast = swiftui_ast::build_ast_from_ui_spec(&spec).map_err(swiftui_ast_build_error)?;
    let rendered = swiftui_codegen::render_swift_file(&ast);

    let output_dir = workspace_root.join(SWIFT_ARTIFACT_OUTPUT_DIR);
    std::fs::create_dir_all(&output_dir).map_err(io_error)?;
    let relative_path = format!("{SWIFT_ARTIFACT_OUTPUT_DIR}/{}.swift", ast.view_name);
    let output_path = workspace_root.join(relative_path.as_str());
    std::fs::write(&output_path, rendered).map_err(io_error)?;

    Ok(relative_path)
}

fn io_error(err: std::io::Error) -> PipelineError {
    PipelineError::Io(err.to_string())
}

fn serialization_error(err: serde_json::Error) -> PipelineError {
    PipelineError::Serialization(err.to_string())
}

fn fetch_error(err: figma_client::FetchClientError) -> PipelineError {
    PipelineError::FetchClient(err.to_string())
}

fn normalizer_error(err: figma_normalizer::NormalizationError) -> PipelineError {
    PipelineError::Normalizer(err.to_string())
}

fn ui_spec_build_error(err: ui_spec::UiSpecBuildError) -> PipelineError {
    PipelineError::UiSpecBuild(err.to_string())
}

fn swiftui_ast_build_error(err: swiftui_ast::SwiftUiAstBuildError) -> PipelineError {
    PipelineError::SwiftUiAstBuild(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

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
        let result = run_stage("report").expect("known stage should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "report",
                output_dir: "output/reports",
                artifact_path: None,
            }
        );
    }

    #[test]
    fn run_stage_fetch_writes_snapshot_artifact() {
        let workspace_root = unique_test_workspace_root("run_stage_fetch_writes_snapshot_artifact");

        let result =
            run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "fetch",
                output_dir: "output/raw",
                artifact_path: Some("output/raw/fetch_snapshot.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/raw/fetch_snapshot.json");
        assert!(artifact_path.is_file(), "fetch artifact should exist");

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let snapshot: figma_client::RawFigmaSnapshot =
            serde_json::from_str(&artifact).expect("artifact should be valid raw snapshot json");
        assert_eq!(
            snapshot.snapshot_version,
            figma_client::RAW_SNAPSHOT_SCHEMA_VERSION
        );
        assert_eq!(snapshot.source.file_key, "fixture-file-key");
        assert_eq!(snapshot.source.node_id, "0:1");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_normalize_reads_raw_and_writes_normalized_artifact() {
        let workspace_root = unique_test_workspace_root(
            "run_stage_normalize_reads_raw_and_writes_normalized_artifact",
        );

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        let result = run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "normalize",
                output_dir: "output/normalized",
                artifact_path: Some("output/normalized/normalized_document.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/normalized/normalized_document.json");
        assert!(artifact_path.is_file(), "normalized artifact should exist");

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let normalized: figma_normalizer::NormalizationOutput =
            serde_json::from_str(&artifact).expect("artifact should be valid normalized json");
        assert_eq!(
            normalized.document.schema_version,
            figma_normalizer::NORMALIZED_SCHEMA_VERSION
        );
        assert_eq!(normalized.document.source.file_key, "fixture-file-key");
        assert_eq!(normalized.document.source.root_node_id, "0:1");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_normalize_requires_raw_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_normalize_requires_raw_artifact");

        let err = run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect_err("normalize should fail without fetch artifact");
        assert_eq!(
            err,
            PipelineError::MissingInputArtifact("output/raw/fetch_snapshot.json".to_string())
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_infer_layout_writes_inferred_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_infer_layout_writes_inferred_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        let result = run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "infer-layout",
                output_dir: "output/inferred",
                artifact_path: Some("output/inferred/layout_inference.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/inferred/layout_inference.json");
        assert!(artifact_path.is_file(), "inferred artifact should exist");

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let inferred: layout_infer::InferredLayoutDocument =
            serde_json::from_str(&artifact).expect("artifact should be valid inferred json");
        assert_eq!(
            inferred.inference_version,
            layout_infer::LAYOUT_DECISION_VERSION
        );
        assert_eq!(inferred.source_file_key, "fixture-file-key");
        assert_eq!(inferred.root_node_id, "0:1");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_build_spec_writes_spec_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_build_spec_writes_spec_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run first");
        let result = run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "build-spec",
                output_dir: "output/specs",
                artifact_path: Some("output/specs/ui_spec.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/specs/ui_spec.json");
        assert!(artifact_path.is_file(), "ui spec artifact should exist");

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let spec: ui_spec::UiSpec =
            serde_json::from_str(&artifact).expect("artifact should be valid ui spec json");
        assert_eq!(spec.spec_version, ui_spec::UI_SPEC_VERSION);
        assert_eq!(spec.source.file_key, "fixture-file-key");
        assert_eq!(spec.source.root_node_id, "0:1");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_gen_swiftui_writes_swift_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_gen_swiftui_writes_swift_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run first");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run first");
        let result = run_stage_in_workspace("gen-swiftui", workspace_root.as_path())
            .expect("gen-swiftui should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "gen-swiftui",
                output_dir: "output/swift",
                artifact_path: Some("output/swift/FixtureRootView.swift".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/swift/FixtureRootView.swift");
        assert!(artifact_path.is_file(), "swift artifact should exist");

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        assert!(artifact.contains("import SwiftUI"));
        assert!(artifact.contains("struct FixtureRootView: View"));

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_returns_error_for_unknown_stage() {
        let err = run_stage("not-a-stage").expect_err("unknown stage should fail");
        assert_eq!(err, PipelineError::UnknownStage("not-a-stage".to_string()));
    }

    fn unique_test_workspace_root(test_name: &str) -> PathBuf {
        let timestamp_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "forge-{test_name}-{}-{timestamp_nanos}",
            std::process::id()
        ));
        ensure_dir(path.as_path());
        path
    }

    fn ensure_dir(path: &Path) {
        std::fs::create_dir_all(path).expect("test workspace root should be created");
    }
}

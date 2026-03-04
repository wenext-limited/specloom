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
}

impl PipelineError {
    pub fn actionable_message(&self) -> String {
        match self {
            Self::UnknownStage(stage) => format!(
                "unknown stage: {stage}. Valid stages: {}. Run `cli stages` to list stage output directories.",
                pipeline_stage_names().join(", ")
            ),
            Self::MissingInputArtifact(artifact_path) => {
                if let Some(stage_name) = producer_stage_for_artifact(artifact_path.as_str()) {
                    format!(
                        "missing input artifact: {artifact_path}. Run `cli run-stage {stage_name}` first, or run `cli generate` to execute the full pipeline."
                    )
                } else {
                    format!(
                        "missing input artifact: {artifact_path}. Run `cli generate` to execute the full pipeline."
                    )
                }
            }
            Self::Io(details) => format!(
                "io error: {details}. Check that the working directory is writable and that `output/` is a directory."
            ),
            Self::Serialization(details) => format!(
                "serialization error: {details}. Delete stale artifacts under `output/` and rerun the upstream stage."
            ),
            Self::FetchClient(details) => format!(
                "fetch client error: {details}. For live fetch, verify `--input live`, `--file-key`, `--node-id`, and `FIGMA_TOKEN` (or `--figma-token`), then confirm file and node permissions in Figma."
            ),
            _ => self.to_string(),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineRunConfig {
    pub fetch_mode: FetchMode,
}

impl Default for PipelineRunConfig {
    fn default() -> Self {
        Self {
            fetch_mode: FetchMode::Fixture,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchMode {
    Fixture,
    Live(LiveFetchConfig),
    Snapshot(SnapshotFetchConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFetchConfig {
    pub file_key: String,
    pub node_id: String,
    pub figma_token: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFetchConfig {
    pub snapshot_path: String,
}

const FETCH_ARTIFACT_RELATIVE_PATH: &str = "output/raw/fetch_snapshot.json";
const NORMALIZED_ARTIFACT_RELATIVE_PATH: &str = "output/normalized/normalized_document.json";
const INFERRED_ARTIFACT_RELATIVE_PATH: &str = "output/inferred/layout_inference.json";
const SPEC_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_spec.ron";
const ASSET_MANIFEST_RELATIVE_PATH: &str = "output/assets/asset_manifest.json";

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

fn producer_stage_for_artifact(artifact_path: &str) -> Option<&'static str> {
    match artifact_path {
        FETCH_ARTIFACT_RELATIVE_PATH => Some("fetch"),
        NORMALIZED_ARTIFACT_RELATIVE_PATH => Some("normalize"),
        INFERRED_ARTIFACT_RELATIVE_PATH => Some("infer-layout"),
        SPEC_ARTIFACT_RELATIVE_PATH => Some("build-spec"),
        ASSET_MANIFEST_RELATIVE_PATH => Some("export-assets"),
        _ => None,
    }
}

const PIPELINE_STAGES: [PipelineStageDefinition; 5] = [
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
        name: "export-assets",
        output_dir: "output/assets",
    },
];

const DEFAULT_RUN_ALL_STAGE_NAMES: [&str; 5] = [
    "fetch",
    "normalize",
    "infer-layout",
    "build-spec",
    "export-assets",
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
    run_stage_with_config(stage_name, &PipelineRunConfig::default())
}

pub fn run_stage_with_config(
    stage_name: &str,
    config: &PipelineRunConfig,
) -> Result<StageExecutionResult, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    run_stage_in_workspace_with_config(stage_name, workspace_root.as_path(), config)
}

pub fn run_all() -> Result<Vec<StageExecutionResult>, PipelineError> {
    run_all_with_config(&PipelineRunConfig::default())
}

pub fn run_all_with_config(
    config: &PipelineRunConfig,
) -> Result<Vec<StageExecutionResult>, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    run_all_in_workspace_with_config(workspace_root.as_path(), config)
}

pub fn run_all_in_workspace(
    workspace_root: &Path,
) -> Result<Vec<StageExecutionResult>, PipelineError> {
    run_all_in_workspace_with_config(workspace_root, &PipelineRunConfig::default())
}

pub fn run_all_in_workspace_with_config(
    workspace_root: &Path,
    config: &PipelineRunConfig,
) -> Result<Vec<StageExecutionResult>, PipelineError> {
    DEFAULT_RUN_ALL_STAGE_NAMES
        .iter()
        .map(|stage_name| run_stage_in_workspace_with_config(stage_name, workspace_root, config))
        .collect()
}

pub fn run_stage_in_workspace(
    stage_name: &str,
    workspace_root: &Path,
) -> Result<StageExecutionResult, PipelineError> {
    run_stage_in_workspace_with_config(stage_name, workspace_root, &PipelineRunConfig::default())
}

pub fn run_stage_in_workspace_with_config(
    stage_name: &str,
    workspace_root: &Path,
    config: &PipelineRunConfig,
) -> Result<StageExecutionResult, PipelineError> {
    let output = match stage_name {
        "fetch" => Some(run_fetch_stage(workspace_root, &config.fetch_mode)?),
        "normalize" => Some(run_normalize_stage(workspace_root)?),
        "infer-layout" => Some(run_infer_layout_stage(workspace_root)?),
        "build-spec" => Some(run_build_spec_stage(workspace_root)?),
        "export-assets" => Some(run_export_assets_stage(workspace_root)?),
        _ => None,
    };

    let stage = PIPELINE_STAGES
        .iter()
        .find(|candidate| candidate.name == stage_name)
        .ok_or_else(|| PipelineError::UnknownStage(stage_name.to_string()))?;

    Ok(StageExecutionResult {
        stage_name: stage.name,
        output_dir: stage.output_dir,
        artifact_path: output,
    })
}

fn run_fetch_stage(workspace_root: &Path, fetch_mode: &FetchMode) -> Result<String, PipelineError> {
    let snapshot = match fetch_mode {
        FetchMode::Fixture => fetch_fixture_snapshot()?,
        FetchMode::Live(config) => fetch_live_snapshot(config)?,
        FetchMode::Snapshot(config) => load_snapshot_from_file(workspace_root, config)?,
    };

    let artifact_path = workspace_root.join(FETCH_ARTIFACT_RELATIVE_PATH);
    write_bytes(
        artifact_path.as_path(),
        serde_json::to_vec_pretty(&snapshot)
            .map_err(serialization_error)?
            .as_slice(),
    )?;

    Ok(normalize_result_path(
        workspace_root,
        artifact_path.as_path(),
    ))
}

fn fetch_fixture_snapshot() -> Result<figma_client::RawFigmaSnapshot, PipelineError> {
    let request = figma_client::FetchNodesRequest::new(
        FETCH_FIXTURE_FILE_KEY.to_string(),
        FETCH_FIXTURE_NODE_ID.to_string(),
    )
    .map_err(fetch_client_error)?;

    figma_client::fetch_snapshot_from_fixture(&request, FETCH_FIXTURE_JSON)
        .map_err(fetch_client_error)
}

fn fetch_live_snapshot(
    config: &LiveFetchConfig,
) -> Result<figma_client::RawFigmaSnapshot, PipelineError> {
    let request = figma_client::LiveFetchRequest::new(
        config.file_key.clone(),
        config.node_id.clone(),
        config.figma_token.clone(),
        config.api_base_url.clone(),
    )
    .map_err(fetch_client_error)?;

    figma_client::fetch_snapshot_live(&request).map_err(fetch_client_error)
}

fn load_snapshot_from_file(
    workspace_root: &Path,
    config: &SnapshotFetchConfig,
) -> Result<figma_client::RawFigmaSnapshot, PipelineError> {
    let snapshot_path = resolve_workspace_path(workspace_root, config.snapshot_path.as_str());
    let bytes = std::fs::read(snapshot_path.as_path()).map_err(io_error)?;
    serde_json::from_slice::<figma_client::RawFigmaSnapshot>(&bytes).map_err(serialization_error)
}

fn resolve_workspace_path(workspace_root: &Path, candidate_path: &str) -> std::path::PathBuf {
    let path = Path::new(candidate_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn run_normalize_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let snapshot = read_required_json::<figma_client::RawFigmaSnapshot>(
        workspace_root,
        FETCH_ARTIFACT_RELATIVE_PATH,
    )?;

    let normalized = figma_normalizer::normalize_snapshot(&snapshot).map_err(normalizer_error)?;
    let output_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    write_bytes(
        output_path.as_path(),
        serde_json::to_vec_pretty(&normalized)
            .map_err(serialization_error)?
            .as_slice(),
    )?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn run_infer_layout_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<figma_normalizer::NormalizationOutput>(
        workspace_root,
        NORMALIZED_ARTIFACT_RELATIVE_PATH,
    )?;

    let inferred = layout_infer::infer_layout(&normalized.document);
    let output_path = workspace_root.join(INFERRED_ARTIFACT_RELATIVE_PATH);
    write_bytes(
        output_path.as_path(),
        serde_json::to_vec_pretty(&inferred)
            .map_err(serialization_error)?
            .as_slice(),
    )?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn run_build_spec_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<figma_normalizer::NormalizationOutput>(
        workspace_root,
        NORMALIZED_ARTIFACT_RELATIVE_PATH,
    )?;
    let inferred = read_required_json::<layout_infer::InferredLayoutDocument>(
        workspace_root,
        INFERRED_ARTIFACT_RELATIVE_PATH,
    )?;

    let spec = ui_spec::build_ui_spec(&normalized, &inferred).map_err(ui_spec_build_error)?;
    let encoded = spec
        .to_pretty_ron()
        .map_err(|err| PipelineError::Serialization(err.to_string()))?;

    let output_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    write_bytes(output_path.as_path(), encoded.as_bytes())?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn run_export_assets_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<figma_normalizer::NormalizationOutput>(
        workspace_root,
        NORMALIZED_ARTIFACT_RELATIVE_PATH,
    )?;

    let assets = asset_pipeline::build_asset_manifest(&normalized);
    let encoded = serde_json::to_vec_pretty(&assets).map_err(serialization_error)?;

    let output_path = workspace_root.join(ASSET_MANIFEST_RELATIVE_PATH);
    write_bytes(output_path.as_path(), encoded.as_slice())?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn read_required_json<T>(workspace_root: &Path, relative_path: &str) -> Result<T, PipelineError>
where
    T: serde::de::DeserializeOwned,
{
    let path = workspace_root.join(relative_path);
    if !path.exists() {
        return Err(PipelineError::MissingInputArtifact(
            relative_path.to_string(),
        ));
    }

    let bytes = std::fs::read(path.as_path()).map_err(io_error)?;
    serde_json::from_slice(&bytes).map_err(serialization_error)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), PipelineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    std::fs::write(path, bytes).map_err(io_error)
}

fn normalize_result_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .ok()
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn io_error(err: std::io::Error) -> PipelineError {
    PipelineError::Io(err.to_string())
}

fn serialization_error(err: impl std::fmt::Display) -> PipelineError {
    PipelineError::Serialization(err.to_string())
}

fn fetch_client_error(err: figma_client::FetchClientError) -> PipelineError {
    PipelineError::FetchClient(err.to_string())
}

fn normalizer_error(err: figma_normalizer::NormalizationError) -> PipelineError {
    PipelineError::Normalizer(err.to_string())
}

fn ui_spec_build_error(err: ui_spec::UiSpecBuildError) -> PipelineError {
    PipelineError::UiSpecBuild(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_are_reported_in_order() {
        assert_eq!(
            pipeline_stage_names(),
            vec![
                "fetch",
                "normalize",
                "infer-layout",
                "build-spec",
                "export-assets"
            ]
        );
    }

    #[test]
    fn stage_output_directories_are_reported() {
        assert_eq!(
            pipeline_stage_output_dirs(),
            vec![
                ("fetch", "output/raw"),
                ("normalize", "output/normalized"),
                ("infer-layout", "output/inferred"),
                ("build-spec", "output/specs"),
                ("export-assets", "output/assets"),
            ]
        );
    }

    #[test]
    fn run_stage_returns_execution_result_for_known_stage() {
        let workspace_root =
            unique_test_workspace_root("run_stage_returns_execution_result_for_known_stage");
        let result = run_stage_in_workspace("fetch", workspace_root.as_path())
            .expect("fetch stage should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "fetch",
                output_dir: "output/raw",
                artifact_path: Some("output/raw/fetch_snapshot.json".to_string()),
            }
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_unknown_stage_is_rejected() {
        let err = run_stage("not-a-stage").expect_err("unknown stage should fail");
        assert_eq!(err, PipelineError::UnknownStage("not-a-stage".to_string()));
        assert!(err.actionable_message().contains("Valid stages:"));
    }

    #[test]
    fn run_stage_fetch_with_snapshot_input_writes_snapshot_artifact() {
        let workspace_root = unique_test_workspace_root(
            "run_stage_fetch_with_snapshot_input_writes_snapshot_artifact",
        );
        let input_snapshot_path = workspace_root.join("fixtures/snapshot.json");
        if let Some(parent) = input_snapshot_path.parent() {
            std::fs::create_dir_all(parent).expect("fixture parent should be creatable");
        }
        std::fs::write(
            input_snapshot_path.as_path(),
            r#"{
                "snapshot_version": "1.0",
                "source": {
                    "file_key": "snapshot-file-key",
                    "node_id": "7:7",
                    "figma_api_version": "v1"
                },
                "payload": {
                    "document": {
                        "id": "7:7",
                        "name": "Snapshot Root",
                        "type": "FRAME",
                        "children": []
                    }
                }
            }"#,
        )
        .expect("fixture snapshot should be written");

        let config = PipelineRunConfig {
            fetch_mode: FetchMode::Snapshot(SnapshotFetchConfig {
                snapshot_path: "fixtures/snapshot.json".to_string(),
            }),
        };

        let result = run_stage_in_workspace_with_config("fetch", workspace_root.as_path(), &config)
            .expect("fetch stage should run");
        assert_eq!(result.stage_name, "fetch");
        assert_eq!(result.output_dir, "output/raw");

        let artifact_path = workspace_root.join("output/raw/fetch_snapshot.json");
        assert!(artifact_path.is_file());

        let artifact = std::fs::read_to_string(artifact_path).expect("snapshot should be readable");
        let snapshot: figma_client::RawFigmaSnapshot =
            serde_json::from_str(&artifact).expect("snapshot should decode");
        assert_eq!(snapshot.source.file_key, "snapshot-file-key");
        assert_eq!(snapshot.source.node_id, "7:7");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_normalize_requires_raw_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_normalize_requires_raw_artifact");

        let err = run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect_err("normalize should fail when raw artifact is missing");

        assert_eq!(
            err,
            PipelineError::MissingInputArtifact("output/raw/fetch_snapshot.json".to_string())
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_build_spec_writes_ron_artifact() {
        let workspace_root = unique_test_workspace_root("run_stage_build_spec_writes_ron_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run third");

        let result = run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "build-spec",
                output_dir: "output/specs",
                artifact_path: Some("output/specs/ui_spec.ron".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/specs/ui_spec.ron");
        assert!(artifact_path.is_file());
        let artifact = std::fs::read_to_string(artifact_path).expect("spec should be readable");
        assert!(artifact.contains("Container("));

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_export_assets_writes_asset_manifest_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_export_assets_writes_asset_manifest_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");

        let result = run_stage_in_workspace("export-assets", workspace_root.as_path())
            .expect("export-assets should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "export-assets",
                output_dir: "output/assets",
                artifact_path: Some("output/assets/asset_manifest.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/assets/asset_manifest.json");
        assert!(artifact_path.is_file());

        let artifact = std::fs::read_to_string(artifact_path).expect("manifest should be readable");
        let manifest: asset_pipeline::AssetManifest =
            serde_json::from_str(&artifact).expect("manifest should decode");
        assert_eq!(
            manifest.manifest_version,
            asset_pipeline::ASSET_MANIFEST_VERSION
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_all_in_workspace_executes_stages_in_order() {
        let workspace_root =
            unique_test_workspace_root("run_all_in_workspace_executes_stages_in_order");

        let results =
            run_all_in_workspace(workspace_root.as_path()).expect("run-all should succeed");

        assert_eq!(
            results
                .iter()
                .map(|result| result.stage_name)
                .collect::<Vec<_>>(),
            vec![
                "fetch",
                "normalize",
                "infer-layout",
                "build-spec",
                "export-assets"
            ]
        );
        assert_eq!(
            results
                .last()
                .and_then(|result| result.artifact_path.clone()),
            Some("output/assets/asset_manifest.json".to_string())
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_fetch_with_live_config_surfaces_actionable_fetch_error() {
        let workspace_root =
            unique_test_workspace_root("run_stage_fetch_with_live_config_surfaces_fetch_error");

        let config = PipelineRunConfig {
            fetch_mode: FetchMode::Live(LiveFetchConfig {
                file_key: "abc123".to_string(),
                node_id: "123:456".to_string(),
                figma_token: "token-from-test".to_string(),
                api_base_url: Some("http://127.0.0.1:9".to_string()),
            }),
        };

        let err = run_stage_in_workspace_with_config("fetch", workspace_root.as_path(), &config)
            .expect_err("live fetch should fail for unreachable endpoint");
        let message = err.actionable_message();
        assert!(message.contains("fetch client error:"));
        assert!(message.contains("For live fetch, verify"));

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    fn unique_test_workspace_root(test_name: &str) -> std::path::PathBuf {
        let timestamp_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "forge-orchestrator-{test_name}-{}-{timestamp_nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(path.as_path()).expect("workspace root should be created");
        path
    }
}

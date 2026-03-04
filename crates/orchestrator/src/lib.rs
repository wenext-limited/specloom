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
    #[error("llm bundle error: {0}")]
    LlmBundle(String),
    #[error("llm code generation error: {0}")]
    LlmCodegen(String),
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
            Self::LlmCodegen(details) => format!(
                "llm code generation error: {details}. Verify `OPENAI_API_KEY` (or `--api-key`), confirm the LLM bundle exists (`output/llm/llm_bundle.json` by default), and rerun `cli generate-ui`."
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFetchConfig {
    pub file_key: String,
    pub node_id: String,
    pub figma_token: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiConfig {
    pub target: String,
    pub model: String,
    pub api_key: String,
    pub bundle_path: Option<String>,
    pub output_dir: Option<String>,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiExecutionResult {
    pub output_dir: String,
    pub run_record_path: String,
    pub written_files: Vec<String>,
}

const FETCH_ARTIFACT_RELATIVE_PATH: &str = "output/raw/fetch_snapshot.json";
const NORMALIZED_ARTIFACT_RELATIVE_PATH: &str = "output/normalized/normalized_document.json";
const INFERRED_ARTIFACT_RELATIVE_PATH: &str = "output/inferred/layout_inference.json";
const SPEC_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_spec.json";
const BLUEPRINT_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_blueprint.yaml";
const SWIFT_ARTIFACT_OUTPUT_DIR: &str = "output/swift";
const ASSET_MANIFEST_RELATIVE_PATH: &str = "output/assets/asset_manifest.json";
const REPORT_ARTIFACT_RELATIVE_PATH: &str = "output/reports/review_report.json";
const LLM_BUNDLE_ARTIFACT_RELATIVE_PATH: &str = "output/llm/llm_bundle.json";
const GENERATED_UI_OUTPUT_RELATIVE_DIR: &str = "output/generated-ui";
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
        BLUEPRINT_ARTIFACT_RELATIVE_PATH => Some("build-ui-blueprint"),
        ASSET_MANIFEST_RELATIVE_PATH => Some("export-assets"),
        REPORT_ARTIFACT_RELATIVE_PATH => Some("report"),
        LLM_BUNDLE_ARTIFACT_RELATIVE_PATH => Some("prepare-llm-bundle"),
        _ => None,
    }
}

const PIPELINE_STAGES: [PipelineStageDefinition; 9] = [
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
        name: "build-ui-blueprint",
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
    PipelineStageDefinition {
        name: "prepare-llm-bundle",
        output_dir: "output/llm",
    },
];

const DEFAULT_RUN_ALL_STAGE_NAMES: [&str; 7] = [
    "fetch",
    "normalize",
    "infer-layout",
    "build-spec",
    "build-ui-blueprint",
    "export-assets",
    "report",
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

pub fn generate_ui(config: &GenerateUiConfig) -> Result<GenerateUiExecutionResult, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    generate_ui_in_workspace(workspace_root.as_path(), config)
}

pub fn generate_ui_in_workspace(
    workspace_root: &Path,
    config: &GenerateUiConfig,
) -> Result<GenerateUiExecutionResult, PipelineError> {
    let bundle_input = config
        .bundle_path
        .as_deref()
        .unwrap_or(LLM_BUNDLE_ARTIFACT_RELATIVE_PATH);
    let output_input = config
        .output_dir
        .as_deref()
        .unwrap_or(GENERATED_UI_OUTPUT_RELATIVE_DIR);

    let bundle_path = resolve_input_path(workspace_root, bundle_input);
    let output_dir = resolve_input_path(workspace_root, output_input);
    let result = llm_codegen::generate_ui(&llm_codegen::GenerateUiRequest {
        model: config.model.clone(),
        target: config.target.clone(),
        bundle_path: bundle_path.clone(),
        output_dir: output_dir.clone(),
        api_key: config.api_key.clone(),
        api_base_url: config.api_base_url.clone(),
    })
    .map_err(llm_codegen_error)?;

    Ok(GenerateUiExecutionResult {
        output_dir: normalize_result_path(workspace_root, output_dir.as_path()),
        run_record_path: normalize_result_path(
            workspace_root,
            Path::new(result.run_record_path.as_str()),
        ),
        written_files: result
            .written_files
            .iter()
            .map(|path| normalize_result_path(workspace_root, Path::new(path.as_str())))
            .collect(),
    })
}

pub fn run_stage_in_workspace_with_config(
    stage_name: &str,
    workspace_root: &Path,
    config: &PipelineRunConfig,
) -> Result<StageExecutionResult, PipelineError> {
    let stage = PIPELINE_STAGES
        .iter()
        .copied()
        .find(|candidate| candidate.name == stage_name)
        .ok_or_else(|| PipelineError::UnknownStage(stage_name.to_string()))?;

    let artifact_path = match stage.name {
        "fetch" => Some(run_fetch_stage(workspace_root, &config.fetch_mode)?),
        "normalize" => Some(run_normalize_stage(workspace_root)?),
        "infer-layout" => Some(run_infer_layout_stage(workspace_root)?),
        "build-spec" => Some(run_build_spec_stage(workspace_root)?),
        "build-ui-blueprint" => Some(run_build_ui_blueprint_stage(workspace_root)?),
        "gen-swiftui" => Some(run_gen_swiftui_stage(workspace_root)?),
        "export-assets" => Some(run_export_assets_stage(workspace_root)?),
        "report" => Some(run_report_stage(workspace_root)?),
        "prepare-llm-bundle" => Some(run_prepare_llm_bundle_stage(workspace_root)?),
        _ => None,
    };

    Ok(StageExecutionResult {
        stage_name: stage.name,
        output_dir: stage.output_dir,
        artifact_path,
    })
}

fn run_fetch_stage(workspace_root: &Path, fetch_mode: &FetchMode) -> Result<String, PipelineError> {
    let snapshot = match fetch_mode {
        FetchMode::Fixture => {
            let request = figma_client::FetchNodesRequest::new(
                FETCH_FIXTURE_FILE_KEY.to_string(),
                FETCH_FIXTURE_NODE_ID.to_string(),
            )
            .map_err(fetch_error)?;
            figma_client::fetch_snapshot_from_fixture(&request, FETCH_FIXTURE_JSON)
                .map_err(fetch_error)?
        }
        FetchMode::Live(config) => {
            let request = figma_client::LiveFetchRequest::new(
                config.file_key.clone(),
                config.node_id.clone(),
                config.figma_token.clone(),
                config.api_base_url.clone(),
            )
            .map_err(fetch_error)?;
            figma_client::fetch_snapshot_live(&request).map_err(fetch_error)?
        }
    };

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

fn run_build_ui_blueprint_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let spec_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    if !spec_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            SPEC_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let spec_artifact = std::fs::read_to_string(&spec_path).map_err(io_error)?;
    let spec: ui_spec::UiSpec =
        serde_json::from_str(&spec_artifact).map_err(serialization_error)?;
    let blueprint = ui_blueprint::build_ui_blueprint(&spec);

    let blueprint_path = workspace_root.join(BLUEPRINT_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = blueprint_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = blueprint.to_yaml_string().map_err(blueprint_yaml_error)?;
    std::fs::write(&blueprint_path, encoded).map_err(io_error)?;

    Ok(BLUEPRINT_ARTIFACT_RELATIVE_PATH.to_string())
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

fn run_export_assets_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    if !normalized_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            NORMALIZED_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }
    let spec_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    if !spec_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            SPEC_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let normalized_artifact = std::fs::read_to_string(&normalized_path).map_err(io_error)?;
    let normalized: figma_normalizer::NormalizationOutput =
        serde_json::from_str(&normalized_artifact).map_err(serialization_error)?;

    let spec_artifact = std::fs::read_to_string(&spec_path).map_err(io_error)?;
    let spec: ui_spec::UiSpec =
        serde_json::from_str(&spec_artifact).map_err(serialization_error)?;

    let manifest = asset_pipeline::build_asset_manifest(&normalized, &spec);

    let output_path = workspace_root.join(ASSET_MANIFEST_RELATIVE_PATH);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = serde_json::to_string_pretty(&manifest).map_err(serialization_error)?;
    std::fs::write(&output_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(ASSET_MANIFEST_RELATIVE_PATH.to_string())
}

fn run_report_stage(workspace_root: &Path) -> Result<String, PipelineError> {
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
    let assets_path = workspace_root.join(ASSET_MANIFEST_RELATIVE_PATH);
    if !assets_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            ASSET_MANIFEST_RELATIVE_PATH.to_string(),
        ));
    }

    let normalized_artifact = std::fs::read_to_string(&normalized_path).map_err(io_error)?;
    let normalized: figma_normalizer::NormalizationOutput =
        serde_json::from_str(&normalized_artifact).map_err(serialization_error)?;

    let inferred_artifact = std::fs::read_to_string(&inferred_path).map_err(io_error)?;
    let inferred: layout_infer::InferredLayoutDocument =
        serde_json::from_str(&inferred_artifact).map_err(serialization_error)?;

    let assets_artifact = std::fs::read_to_string(&assets_path).map_err(io_error)?;
    let assets: asset_pipeline::AssetManifest =
        serde_json::from_str(&assets_artifact).map_err(serialization_error)?;

    let report =
        review_report::build_review_report(&normalized.warnings, &inferred, &assets.warnings);

    let output_path = workspace_root.join(REPORT_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let encoded = serde_json::to_string_pretty(&report).map_err(serialization_error)?;
    std::fs::write(&output_path, format!("{encoded}\n")).map_err(io_error)?;

    Ok(REPORT_ARTIFACT_RELATIVE_PATH.to_string())
}

fn run_prepare_llm_bundle_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let blueprint_path = workspace_root.join(BLUEPRINT_ARTIFACT_RELATIVE_PATH);
    if !blueprint_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            BLUEPRINT_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }
    let assets_path = workspace_root.join(ASSET_MANIFEST_RELATIVE_PATH);
    if !assets_path.is_file() {
        return Err(PipelineError::MissingInputArtifact(
            ASSET_MANIFEST_RELATIVE_PATH.to_string(),
        ));
    }

    let blueprint_artifact = std::fs::read_to_string(&blueprint_path).map_err(io_error)?;
    let blueprint: ui_blueprint::UiBlueprint =
        serde_yaml::from_str(&blueprint_artifact).map_err(yaml_deserialization_error)?;
    let warnings_summary = blueprint
        .warnings
        .iter()
        .map(|warning| llm_bundle::BundleWarningSummary {
            code: warning.code.clone(),
            node_id: warning.node_id.clone(),
        })
        .collect::<Vec<_>>();

    let mut bundle = llm_bundle::build_bundle(
        "target-agnostic",
        blueprint_path.as_path(),
        assets_path.as_path(),
        warnings_summary,
        "v1",
    )
    .map_err(llm_bundle_error)?;
    bundle.blueprint.path = BLUEPRINT_ARTIFACT_RELATIVE_PATH.to_string();
    bundle.asset_manifest.path = ASSET_MANIFEST_RELATIVE_PATH.to_string();

    let output_path = workspace_root.join(LLM_BUNDLE_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error)?;
    }
    let mut encoded = bundle.to_pretty_json().map_err(serialization_error)?;
    encoded.push(b'\n');
    std::fs::write(&output_path, encoded).map_err(io_error)?;

    Ok(LLM_BUNDLE_ARTIFACT_RELATIVE_PATH.to_string())
}

fn resolve_input_path(workspace_root: &Path, input: &str) -> std::path::PathBuf {
    let path = Path::new(input);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn normalize_result_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .map(|relative| relative.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
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

fn blueprint_yaml_error(err: ui_blueprint::BlueprintError) -> PipelineError {
    PipelineError::Serialization(err.to_string())
}

fn yaml_deserialization_error(err: serde_yaml::Error) -> PipelineError {
    PipelineError::Serialization(err.to_string())
}

fn llm_bundle_error(err: llm_bundle::LlmBundleError) -> PipelineError {
    PipelineError::LlmBundle(err.to_string())
}

fn llm_codegen_error(err: llm_codegen::GenerateUiError) -> PipelineError {
    PipelineError::LlmCodegen(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
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
                "build-ui-blueprint",
                "gen-swiftui",
                "export-assets",
                "report",
                "prepare-llm-bundle",
            ]
        );
    }

    #[test]
    fn unsupported_feature_is_classified() {
        let err = PipelineError::UnsupportedFeature("mask".to_string());
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn unknown_stage_actionable_message_lists_valid_stages() {
        let message = PipelineError::UnknownStage("not-a-stage".to_string()).actionable_message();
        assert!(message.contains("unknown stage: not-a-stage"));
        assert!(message.contains("Valid stages:"));
        assert!(message.contains("fetch"));
        assert!(message.contains("Run `cli stages`"));
    }

    #[test]
    fn missing_artifact_actionable_message_suggests_upstream_stage() {
        let message =
            PipelineError::MissingInputArtifact("output/raw/fetch_snapshot.json".to_string())
                .actionable_message();
        assert!(message.contains("missing input artifact"));
        assert!(message.contains("run-stage fetch"));
        assert!(message.contains("cli generate"));
    }

    #[test]
    fn io_actionable_message_mentions_writable_workspace() {
        let message = PipelineError::Io("Not a directory".to_string()).actionable_message();
        assert!(message.contains("io error"));
        assert!(message.contains("working directory is writable"));
    }

    #[test]
    fn fetch_client_actionable_message_mentions_token_and_permissions() {
        let message =
            PipelineError::FetchClient("figma api unauthorized".to_string()).actionable_message();
        assert!(message.contains("figma api unauthorized"));
        assert!(message.contains("FIGMA_TOKEN"));
        assert!(message.contains("file and node permissions"));
    }

    #[test]
    fn fetch_client_actionable_message_mentions_live_parameter_hints() {
        let message = PipelineError::FetchClient(
            "invalid figma api response: missing nodes.123:456.document in figma response"
                .to_string(),
        )
        .actionable_message();
        assert!(message.contains("missing nodes.123:456.document"));
        assert!(message.contains("--file-key"));
        assert!(message.contains("--node-id"));
        assert!(message.contains("--input live"));
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
                ("build-ui-blueprint", "output/specs"),
                ("gen-swiftui", "output/swift"),
                ("export-assets", "output/assets"),
                ("report", "output/reports"),
                ("prepare-llm-bundle", "output/llm"),
            ]
        );
    }

    #[test]
    fn run_stage_returns_execution_result_for_known_stage() {
        let workspace_root =
            unique_test_workspace_root("run_stage_returns_execution_result_for_known_stage");
        let result = run_stage_in_workspace("fetch", workspace_root.as_path())
            .expect("known stage should run");
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
    fn run_stage_fetch_with_live_config_writes_snapshot_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_fetch_with_live_config_writes_snapshot_artifact");
        let (base_url, request_rx, server_thread) = start_single_response_server(
            "200 OK",
            r#"{
                "nodes": {
                    "123:456": {
                        "document": {
                            "id": "123:456",
                            "name": "Live Root",
                            "type": "FRAME",
                            "children": []
                        }
                    }
                }
            }"#,
        );
        let config = PipelineRunConfig {
            fetch_mode: FetchMode::Live(LiveFetchConfig {
                file_key: "live-file-key".to_string(),
                node_id: "123:456".to_string(),
                figma_token: "secret-token".to_string(),
                api_base_url: Some(base_url),
            }),
        };

        let result = run_stage_in_workspace_with_config("fetch", workspace_root.as_path(), &config)
            .expect("live fetch should run");
        let raw_request = request_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("mock server should receive request");
        server_thread.join().expect("server thread should finish");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "fetch",
                output_dir: "output/raw",
                artifact_path: Some("output/raw/fetch_snapshot.json".to_string()),
            }
        );

        let lower_request = raw_request.to_ascii_lowercase();
        assert!(
            raw_request.starts_with("GET /v1/files/live-file-key/nodes?ids=123%3A456 HTTP/1.1")
        );
        assert!(lower_request.contains("x-figma-token: secret-token"));

        let artifact_path = workspace_root.join("output/raw/fetch_snapshot.json");
        assert!(artifact_path.is_file(), "fetch artifact should exist");
        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let snapshot: figma_client::RawFigmaSnapshot =
            serde_json::from_str(&artifact).expect("artifact should be valid raw snapshot json");
        assert_eq!(snapshot.source.file_key, "live-file-key");
        assert_eq!(snapshot.source.node_id, "123:456");
        assert_eq!(
            snapshot.payload,
            serde_json::json!({
                "document": {
                    "id": "123:456",
                    "name": "Live Root",
                    "type": "FRAME",
                    "children": []
                }
            })
        );

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
    fn run_stage_build_ui_blueprint_writes_yaml_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_build_ui_blueprint_writes_yaml_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run first");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run first");
        let result = run_stage_in_workspace("build-ui-blueprint", workspace_root.as_path())
            .expect("build-ui-blueprint should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "build-ui-blueprint",
                output_dir: "output/specs",
                artifact_path: Some("output/specs/ui_blueprint.yaml".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/specs/ui_blueprint.yaml");
        assert!(
            artifact_path.is_file(),
            "ui blueprint artifact should exist"
        );

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        assert!(artifact.contains("version: ui_blueprint/1.0"));
        assert!(artifact.contains("file_key: fixture-file-key"));
        assert!(artifact.contains("root_node_id: 0:1"));

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
    fn run_stage_export_assets_writes_asset_manifest_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_export_assets_writes_asset_manifest_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run first");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run first");
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
        assert!(
            artifact_path.is_file(),
            "asset manifest artifact should exist"
        );

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let manifest: asset_pipeline::AssetManifest =
            serde_json::from_str(&artifact).expect("artifact should be valid manifest json");
        assert_eq!(
            manifest.manifest_version,
            asset_pipeline::ASSET_MANIFEST_VERSION
        );
        assert_eq!(manifest.generation.source_file_key, "fixture-file-key");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_stage_prepare_llm_bundle_writes_bundle_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_prepare_llm_bundle_writes_bundle_artifact");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run first");
        run_stage_in_workspace("infer-layout", workspace_root.as_path())
            .expect("infer-layout should run first");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run first");
        run_stage_in_workspace("build-ui-blueprint", workspace_root.as_path())
            .expect("build-ui-blueprint should run first");
        run_stage_in_workspace("export-assets", workspace_root.as_path())
            .expect("export-assets should run first");

        let result = run_stage_in_workspace("prepare-llm-bundle", workspace_root.as_path())
            .expect("prepare-llm-bundle should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "prepare-llm-bundle",
                output_dir: "output/llm",
                artifact_path: Some("output/llm/llm_bundle.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/llm/llm_bundle.json");
        assert!(artifact_path.is_file(), "llm bundle artifact should exist");
        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let bundle: llm_bundle::LlmBundle =
            serde_json::from_str(&artifact).expect("artifact should be valid llm bundle json");
        assert_eq!(bundle.bundle_version, llm_bundle::LLM_BUNDLE_VERSION);
        assert_eq!(bundle.target, "target-agnostic");
        assert_eq!(bundle.prompt_template_version, "v1");
        assert_eq!(bundle.blueprint.path, "output/specs/ui_blueprint.yaml");
        assert_eq!(
            bundle.asset_manifest.path,
            "output/assets/asset_manifest.json"
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
                "build-ui-blueprint",
                "export-assets",
                "report",
            ]
        );
        assert_eq!(
            results
                .last()
                .and_then(|result| result.artifact_path.clone()),
            Some("output/reports/review_report.json".to_string())
        );

        assert!(
            workspace_root
                .join("output/reports/review_report.json")
                .is_file(),
            "report artifact should exist after run-all"
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn run_all_in_workspace_returns_stage_error() {
        let blocked_path = std::env::temp_dir().join(format!(
            "forge-run-all-blocked-path-{}-{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&blocked_path, "blocked").expect("blocked path file should be created");

        let err = run_all_in_workspace(blocked_path.as_path())
            .expect_err("run-all should fail when workspace root is a file path");
        assert!(
            matches!(err, PipelineError::Io(_)),
            "expected io error, got: {err:?}"
        );

        let _ = std::fs::remove_file(&blocked_path);
    }

    #[test]
    fn run_stage_report_writes_review_report_artifact() {
        let workspace_root =
            unique_test_workspace_root("run_stage_report_writes_review_report_artifact");

        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "fixture-file-key".to_string(),
                    root_node_id: "0:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: Vec::new(),
            },
            warnings: vec![figma_normalizer::NormalizationWarning {
                code: "UNSUPPORTED_NODE_FIELD".to_string(),
                message: "unsupported field `clipsContent` ignored during normalization"
                    .to_string(),
                node_id: Some("0:1".to_string()),
            }],
        };
        let normalized_value =
            serde_json::to_value(&normalized).expect("normalized artifact should serialize");
        write_json_artifact(
            workspace_root.as_path(),
            "output/normalized/normalized_document.json",
            &normalized_value,
        );

        let inferred = layout_infer::InferredLayoutDocument {
            inference_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
            source_file_key: "fixture-file-key".to_string(),
            root_node_id: "0:1".to_string(),
            decisions: vec![layout_infer::NodeLayoutDecision {
                node_id: "0:1".to_string(),
                record: layout_infer::LayoutDecisionRecord {
                    decision_version: layout_infer::LAYOUT_DECISION_VERSION.to_string(),
                    selected_strategy: layout_infer::LayoutStrategy::VStack,
                    confidence: 0.62,
                    rationale: "ambiguous child geometry".to_string(),
                    alternatives: Vec::new(),
                    warnings: vec![layout_infer::InferenceWarning {
                        code: layout_infer::WARNING_LOW_CONFIDENCE_GEOMETRY.to_string(),
                        severity: layout_infer::WarningSeverity::Medium,
                        message: "Ambiguous geometry".to_string(),
                        node_id: Some("0:1".to_string()),
                    }],
                },
            }],
        };
        let inferred_value =
            serde_json::to_value(&inferred).expect("inferred artifact should serialize");
        write_json_artifact(
            workspace_root.as_path(),
            "output/inferred/layout_inference.json",
            &inferred_value,
        );

        let manifest = asset_pipeline::AssetManifest {
            manifest_version: asset_pipeline::ASSET_MANIFEST_VERSION.to_string(),
            generation: asset_pipeline::GenerationMetadata {
                source_file_key: "fixture-file-key".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            assets: Vec::new(),
            warnings: vec![asset_pipeline::AssetExportWarning {
                code: "MISSING_IMAGE_REF".to_string(),
                message: "Image fill had no image_ref and was skipped.".to_string(),
                node_id: Some("2:2".to_string()),
                fallback_applied: false,
            }],
        };
        let manifest_value =
            serde_json::to_value(&manifest).expect("asset manifest should serialize");
        write_json_artifact(
            workspace_root.as_path(),
            "output/assets/asset_manifest.json",
            &manifest_value,
        );

        let result =
            run_stage_in_workspace("report", workspace_root.as_path()).expect("report should run");

        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "report",
                output_dir: "output/reports",
                artifact_path: Some("output/reports/review_report.json".to_string()),
            }
        );

        let artifact_path = workspace_root.join("output/reports/review_report.json");
        assert!(
            artifact_path.is_file(),
            "review report artifact should exist"
        );

        let artifact =
            std::fs::read_to_string(&artifact_path).expect("artifact should be readable");
        let report: review_report::ReviewReport =
            serde_json::from_str(&artifact).expect("artifact should be valid report json");
        assert_eq!(report.report_version, "1.0");
        assert_eq!(report.summary.total_warnings, 3);
        assert_eq!(
            report
                .warnings
                .iter()
                .map(|warning| warning.code.clone())
                .collect::<Vec<_>>(),
            vec![
                "UNSUPPORTED_NODE_FIELD".to_string(),
                "LOW_CONFIDENCE_GEOMETRY".to_string(),
                "MISSING_IMAGE_REF".to_string(),
            ]
        );

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

    fn write_json_artifact(
        workspace_root: &Path,
        relative_path: &str,
        artifact: &serde_json::Value,
    ) {
        let output_path = workspace_root.join(relative_path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).expect("artifact parent directory should exist");
        }
        let encoded = serde_json::to_string_pretty(artifact).expect("artifact should serialize");
        std::fs::write(&output_path, format!("{encoded}\n")).expect("artifact should be written");
    }

    fn start_single_response_server(
        status_line: &str,
        body: &str,
    ) -> (
        String,
        std::sync::mpsc::Receiver<String>,
        std::thread::JoinHandle<()>,
    ) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
        let address = listener
            .local_addr()
            .expect("mock server should expose local address");
        let (request_tx, request_rx) = std::sync::mpsc::channel::<String>();
        let status_line = status_line.to_string();
        let body = body.to_string();

        let server_thread = std::thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("mock server should accept one request");
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .expect("mock server should set read timeout");

            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("mock server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buffer[..bytes_read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request = String::from_utf8_lossy(&request_bytes).to_string();
            let _ = request_tx.send(request);

            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}",
                content_length = body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("mock server should write response");
            stream.flush().expect("mock server should flush response");
        });

        (format!("http://{address}"), request_rx, server_thread)
    }
}

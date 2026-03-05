#![forbid(unsafe_code)]

use std::path::Path;

mod agent_context;
mod agent_runner;
mod asset_pipeline;
pub mod figma_client;
mod hash;
mod llm_bundle;
mod ui_spec;
pub use agent_runner::{
    AgentGeneratedFile, AgentRunner, AgentRunnerOutput, AgentRunnerRequest, MockAgentRunner,
};
pub use llm_bundle::{
    BundleArtifactRef, BundleArtifacts, BundleFigmaContext, BundleInstructions, BundleRequest,
    BundleSkillDoc, BundleToolContract, BundleToolDefinition, LLM_BUNDLE_VERSION, LlmBundle,
};

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
                "unknown stage: {stage}. Valid stages: {}. Run `specloom stages` to list stage output directories.",
                pipeline_stage_names().join(", ")
            ),
            Self::MissingInputArtifact(artifact_path) => {
                if let Some(stage_name) = producer_stage_for_artifact(artifact_path.as_str()) {
                    format!(
                        "missing input artifact: {artifact_path}. Run `specloom run-stage {stage_name}` first, or run `specloom generate` to execute the full pipeline."
                    )
                } else {
                    format!(
                        "missing input artifact: {artifact_path}. Run `specloom generate` to execute the full pipeline."
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindNodesStatus {
    Ok,
    LowConfidence,
    NoMatch,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FindNodeCandidate {
    pub node_id: String,
    pub score: f32,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FindNodesResult {
    pub status: FindNodesStatus,
    pub candidates: Vec<FindNodeCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeInfoStatus {
    Ok,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub name: String,
    pub node_type: String,
    pub path: String,
    pub raw_tokens: Vec<String>,
    pub normalized_tokens: Vec<String>,
    pub aliases: Vec<String>,
    pub geometry_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NodeInfoResult {
    pub status: NodeInfoStatus,
    pub node: Option<NodeInfo>,
}

const FETCH_ARTIFACT_RELATIVE_PATH: &str = "output/raw/fetch_snapshot.json";
const NORMALIZED_ARTIFACT_RELATIVE_PATH: &str = "output/normalized/normalized_document.json";
const SPEC_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_spec.ron";
const PRE_LAYOUT_ARTIFACT_RELATIVE_PATH: &str = "output/specs/pre_layout.ron";
const NODE_MAP_ARTIFACT_RELATIVE_PATH: &str = "output/specs/node_map.json";
const TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH: &str = "output/specs/transform_plan.json";
const AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH: &str = "output/agent/agent_context.json";
const SEARCH_INDEX_ARTIFACT_RELATIVE_PATH: &str = "output/agent/search_index.json";
const LLM_BUNDLE_ARTIFACT_RELATIVE_PATH: &str = "output/agent/llm_bundle.json";
const GENERATION_WARNINGS_ARTIFACT_RELATIVE_PATH: &str = "output/reports/generation_warnings.json";
const GENERATION_TRACE_ARTIFACT_RELATIVE_PATH: &str = "output/reports/generation_trace.json";
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
        SPEC_ARTIFACT_RELATIVE_PATH => Some("build-spec"),
        PRE_LAYOUT_ARTIFACT_RELATIVE_PATH => Some("build-spec"),
        NODE_MAP_ARTIFACT_RELATIVE_PATH => Some("build-spec"),
        TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH => Some("build-spec"),
        AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH => Some("build-agent-context"),
        SEARCH_INDEX_ARTIFACT_RELATIVE_PATH => Some("build-agent-context"),
        LLM_BUNDLE_ARTIFACT_RELATIVE_PATH => Some("prepare-llm-bundle"),
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
        name: "build-spec",
        output_dir: "output/specs",
    },
    PipelineStageDefinition {
        name: "build-agent-context",
        output_dir: "output/agent",
    },
    PipelineStageDefinition {
        name: "export-assets",
        output_dir: "output/assets",
    },
];

const DEFAULT_RUN_ALL_STAGE_NAMES: [&str; 5] = [
    "fetch",
    "normalize",
    "build-spec",
    "build-agent-context",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareLlmBundleRequest {
    pub figma_url: String,
    pub target: String,
    pub intent: String,
}

pub fn prepare_llm_bundle(request: &PrepareLlmBundleRequest) -> Result<String, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    prepare_llm_bundle_in_workspace(workspace_root.as_path(), request)
}

pub fn prepare_llm_bundle_in_workspace(
    workspace_root: &Path,
    request: &PrepareLlmBundleRequest,
) -> Result<String, PipelineError> {
    let snapshot = read_required_json::<figma_client::RawFigmaSnapshot>(
        workspace_root,
        FETCH_ARTIFACT_RELATIVE_PATH,
    )?;
    let context = read_required_json::<agent_context::AgentContext>(
        workspace_root,
        AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH,
    )?;

    let instructions = build_bundle_instructions(workspace_root)?;
    let bundle = LlmBundle {
        version: LLM_BUNDLE_VERSION.to_string(),
        request: BundleRequest {
            target: request.target.clone(),
            intent: request.intent.clone(),
        },
        figma: BundleFigmaContext {
            source_url: request.figma_url.clone(),
            file_key: snapshot.source.file_key,
            root_node_id: context.screen.root_node_id.clone(),
        },
        artifacts: BundleArtifacts {
            ui_spec: build_artifact_ref(workspace_root, SPEC_ARTIFACT_RELATIVE_PATH)?,
            agent_context: build_artifact_ref(workspace_root, AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH)?,
            search_index: build_artifact_ref(workspace_root, SEARCH_INDEX_ARTIFACT_RELATIVE_PATH)?,
            asset_manifest: build_artifact_ref(workspace_root, ASSET_MANIFEST_RELATIVE_PATH)?,
            root_screenshot: build_optional_artifact_ref(
                workspace_root,
                context.screen.root_screenshot_ref.as_str(),
            )?,
        },
        instructions,
        tool_contract: BundleToolContract {
            tools: vec![
                BundleToolDefinition {
                    name: "find_nodes".to_string(),
                    usage: "specloom agent-tool find-nodes --query \"<text>\" --output json"
                        .to_string(),
                },
                BundleToolDefinition {
                    name: "get_node_info".to_string(),
                    usage: "specloom agent-tool get-node-info --node-id <NODE_ID> --output json"
                        .to_string(),
                },
                BundleToolDefinition {
                    name: "get_node_screenshot".to_string(),
                    usage: "specloom agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID> --output json".to_string(),
                },
                BundleToolDefinition {
                    name: "get_asset".to_string(),
                    usage: "specloom agent-tool get-asset --node-id <NODE_ID> --output json"
                        .to_string(),
                },
            ],
        },
    };

    let bundle_path = workspace_root.join(LLM_BUNDLE_ARTIFACT_RELATIVE_PATH);
    let bundle_bytes = serde_json::to_vec_pretty(&bundle).map_err(serialization_error)?;
    write_bytes(bundle_path.as_path(), bundle_bytes.as_slice())?;
    Ok(normalize_result_path(workspace_root, bundle_path.as_path()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiRequest {
    pub bundle_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiResult {
    pub generated_paths: Vec<String>,
}

pub fn generate_ui(request: &GenerateUiRequest) -> Result<GenerateUiResult, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    let runner = MockAgentRunner;
    generate_ui_in_workspace(workspace_root.as_path(), request, &runner)
}

pub fn generate_ui_in_workspace(
    workspace_root: &Path,
    request: &GenerateUiRequest,
    runner: &dyn AgentRunner,
) -> Result<GenerateUiResult, PipelineError> {
    ensure_generation_reports_exist(workspace_root)?;

    if request.bundle_path.trim().is_empty() {
        return Err(PipelineError::MissingInputArtifact(
            LLM_BUNDLE_ARTIFACT_RELATIVE_PATH.to_string(),
        ));
    }

    let bundle_path = resolve_workspace_path(workspace_root, request.bundle_path.as_str());
    if !bundle_path.exists() {
        return Err(PipelineError::MissingInputArtifact(
            request.bundle_path.clone(),
        ));
    }

    let bundle_text = std::fs::read_to_string(bundle_path.as_path()).map_err(io_error)?;
    let bundle =
        serde_json::from_str::<LlmBundle>(bundle_text.as_str()).map_err(serialization_error)?;
    let runner_output = match runner.generate(&AgentRunnerRequest { bundle }) {
        Ok(output) => output,
        Err(err) => {
            append_warning(
                workspace_root,
                "GENERATION_RUNNER_FAILURE",
                request.bundle_path.as_str(),
                Vec::new(),
                "stop_generation",
                err.to_string().as_str(),
            )?;
            append_trace_event(
                workspace_root,
                "generate_ui",
                "error",
                request.bundle_path.as_str(),
                Vec::new(),
            )?;
            return Err(err);
        }
    };

    let mut generated_paths = Vec::with_capacity(runner_output.generated_files.len());
    for generated_file in runner_output.generated_files {
        let output_path =
            resolve_workspace_path(workspace_root, generated_file.relative_path.as_str());
        write_bytes(output_path.as_path(), generated_file.contents.as_bytes())?;
        generated_paths.push(normalize_result_path(workspace_root, output_path.as_path()));
    }

    append_trace_event(
        workspace_root,
        "generate_ui",
        "ok",
        request.bundle_path.as_str(),
        generated_paths.clone(),
    )?;

    Ok(GenerateUiResult { generated_paths })
}

pub fn find_nodes(query: &str, top_k: usize) -> Result<FindNodesResult, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    find_nodes_in_workspace(workspace_root.as_path(), query, top_k)
}

pub fn find_nodes_in_workspace(
    workspace_root: &Path,
    query: &str,
    top_k: usize,
) -> Result<FindNodesResult, PipelineError> {
    let search_index = read_required_json::<agent_context::SearchIndex>(
        workspace_root,
        SEARCH_INDEX_ARTIFACT_RELATIVE_PATH,
    )?;
    let result = agent_context::rank_candidates(query, search_index.entries.as_slice(), top_k);

    let status = map_search_status(result.status);
    let candidates = result
        .matches
        .into_iter()
        .map(|candidate| FindNodeCandidate {
            node_id: candidate.node_id,
            score: candidate.score,
            match_reasons: candidate.match_reasons,
        })
        .collect::<Vec<_>>();

    let candidate_node_ids = candidates
        .iter()
        .map(|candidate| candidate.node_id.clone())
        .collect::<Vec<_>>();
    append_trace_event(
        workspace_root,
        "find_nodes",
        find_nodes_status_label(&status),
        query,
        candidate_node_ids.clone(),
    )?;

    match status {
        FindNodesStatus::NoMatch => {
            append_warning(
                workspace_root,
                "NODE_NOT_FOUND",
                query,
                Vec::new(),
                "continue_with_best_effort",
                "No node candidate found for query",
            )?;
        }
        FindNodesStatus::LowConfidence => {
            append_warning(
                workspace_root,
                "LOW_CONFIDENCE_MATCH",
                query,
                candidate_node_ids.clone(),
                "continue_with_best_effort",
                "Top node candidate confidence is below threshold",
            )?;
        }
        FindNodesStatus::Ambiguous => {
            append_warning(
                workspace_root,
                "MULTIPLE_CANDIDATES",
                query,
                candidate_node_ids.clone(),
                "continue_with_best_effort",
                "Multiple close candidates found for query",
            )?;
        }
        FindNodesStatus::Ok => {}
    }

    Ok(FindNodesResult { status, candidates })
}

pub fn get_node_info(node_id: &str) -> Result<NodeInfoResult, PipelineError> {
    let workspace_root = std::env::current_dir().map_err(io_error)?;
    get_node_info_in_workspace(workspace_root.as_path(), node_id)
}

pub fn get_node_info_in_workspace(
    workspace_root: &Path,
    node_id: &str,
) -> Result<NodeInfoResult, PipelineError> {
    let search_index = read_required_json::<agent_context::SearchIndex>(
        workspace_root,
        SEARCH_INDEX_ARTIFACT_RELATIVE_PATH,
    )?;

    let maybe_entry = search_index
        .entries
        .into_iter()
        .find(|entry| entry.node_id == node_id);
    if let Some(entry) = maybe_entry {
        append_trace_event(
            workspace_root,
            "get_node_info",
            node_info_status_label(&NodeInfoStatus::Ok),
            node_id,
            vec![entry.node_id.clone()],
        )?;
        return Ok(NodeInfoResult {
            status: NodeInfoStatus::Ok,
            node: Some(NodeInfo {
                node_id: entry.node_id,
                name: entry.name,
                node_type: entry.node_type,
                path: entry.path,
                raw_tokens: entry.raw_tokens,
                normalized_tokens: entry.normalized_tokens,
                aliases: entry.aliases,
                geometry_tags: entry.geometry_tags,
            }),
        });
    }

    append_trace_event(
        workspace_root,
        "get_node_info",
        node_info_status_label(&NodeInfoStatus::NotFound),
        node_id,
        Vec::new(),
    )?;
    append_warning(
        workspace_root,
        "NODE_NOT_FOUND",
        node_id,
        Vec::new(),
        "continue_with_best_effort",
        "Node ID was not found in search index",
    )?;

    Ok(NodeInfoResult {
        status: NodeInfoStatus::NotFound,
        node: None,
    })
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
        "build-spec" => Some(run_build_spec_stage(workspace_root)?),
        "build-agent-context" => Some(run_build_agent_context_stage(workspace_root, config)?),
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

    let normalized =
        crate::figma_client::normalizer::normalize_snapshot(&snapshot).map_err(normalizer_error)?;
    let output_path = workspace_root.join(NORMALIZED_ARTIFACT_RELATIVE_PATH);
    write_bytes(
        output_path.as_path(),
        serde_json::to_vec_pretty(&normalized)
            .map_err(serialization_error)?
            .as_slice(),
    )?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn run_build_spec_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<crate::figma_client::normalizer::NormalizationOutput>(
        workspace_root,
        NORMALIZED_ARTIFACT_RELATIVE_PATH,
    )?;
    let pre_layout = ui_spec::build_pre_layout_spec(&normalized).map_err(ui_spec_build_error)?;
    let pre_layout_encoded = pre_layout
        .to_pretty_ron()
        .map_err(|err| PipelineError::Serialization(err.to_string()))?;

    let pre_layout_path = workspace_root.join(PRE_LAYOUT_ARTIFACT_RELATIVE_PATH);
    write_bytes(pre_layout_path.as_path(), pre_layout_encoded.as_bytes())?;

    let node_map_path = workspace_root.join(NODE_MAP_ARTIFACT_RELATIVE_PATH);
    let node_map = build_node_map_artifact(&normalized).map_err(serialization_error)?;
    let node_map_bytes = serde_json::to_vec_pretty(&node_map).map_err(serialization_error)?;
    write_bytes(node_map_path.as_path(), node_map_bytes.as_slice())?;

    let transform_plan = generate_transform_plan(workspace_root, &pre_layout, &node_map)?;
    let transform_plan_path = workspace_root.join(TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH);
    let transform_plan_bytes =
        serde_json::to_vec_pretty(&transform_plan).map_err(serialization_error)?;
    write_bytes(
        transform_plan_path.as_path(),
        transform_plan_bytes.as_slice(),
    )?;

    let spec =
        ui_spec::apply_transform_plan(&pre_layout, &transform_plan).map_err(ui_spec_build_error)?;
    let encoded = spec
        .to_pretty_ron()
        .map_err(|err| PipelineError::Serialization(err.to_string()))?;

    let output_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    write_bytes(output_path.as_path(), encoded.as_bytes())?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeMapArtifact {
    version: String,
    nodes: std::collections::BTreeMap<String, serde_json::Value>,
}

fn build_node_map_artifact(
    normalized: &crate::figma_client::normalizer::NormalizationOutput,
) -> Result<NodeMapArtifact, serde_json::Error> {
    let mut nodes = std::collections::BTreeMap::new();
    for node in &normalized.document.nodes {
        nodes.insert(node.id.clone(), serde_json::to_value(node)?);
    }

    Ok(NodeMapArtifact {
        version: "node_map/1.0".to_string(),
        nodes,
    })
}

fn generate_transform_plan(
    workspace_root: &Path,
    _pre_layout: &ui_spec::UiSpec,
    _node_map: &NodeMapArtifact,
) -> Result<ui_spec::TransformPlan, PipelineError> {
    let transform_plan_path = workspace_root.join(TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH);
    if transform_plan_path.exists() {
        let bytes = std::fs::read(transform_plan_path.as_path()).map_err(io_error)?;
        return serde_json::from_slice::<ui_spec::TransformPlan>(bytes.as_slice())
            .map_err(serialization_error);
    }

    Ok(ui_spec::TransformPlan::default())
}

fn run_build_agent_context_stage(
    workspace_root: &Path,
    config: &PipelineRunConfig,
) -> Result<String, PipelineError> {
    let spec = read_required_ron::<ui_spec::UiSpec>(workspace_root, SPEC_ARTIFACT_RELATIVE_PATH)?;

    let root_node_id = spec.id().to_string();
    let root_screenshot_ref = format!("output/images/root_{}.png", root_node_id.replace(':', "_"));

    maybe_write_root_screenshot(
        workspace_root,
        config,
        root_node_id.as_str(),
        root_screenshot_ref.as_str(),
    )?;

    let context = agent_context::AgentContext {
        version: "agent_context/1.0".to_string(),
        screen: agent_context::ScreenRef {
            root_node_id: root_node_id.clone(),
            root_screenshot_ref,
        },
        rules: agent_context::GenerationRules {
            on_node_mismatch: "warn_and_continue".to_string(),
        },
        tools: vec![
            "find_nodes".to_string(),
            "get_node_info".to_string(),
            "get_node_screenshot".to_string(),
            "get_asset".to_string(),
        ],
        skeleton: build_skeleton_nodes(&spec),
    };

    let search_index = agent_context::SearchIndex {
        version: "search_index/1.0".to_string(),
        entries: build_search_index_entries(&spec),
    };

    let context_path = workspace_root.join(AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH);
    let context_bytes = context.to_pretty_json().map_err(serialization_error)?;
    write_bytes(context_path.as_path(), context_bytes.as_slice())?;

    let index_path = workspace_root.join(SEARCH_INDEX_ARTIFACT_RELATIVE_PATH);
    let index_bytes = serde_json::to_vec_pretty(&search_index).map_err(serialization_error)?;
    write_bytes(index_path.as_path(), index_bytes.as_slice())?;

    Ok(normalize_result_path(
        workspace_root,
        context_path.as_path(),
    ))
}

fn maybe_write_root_screenshot(
    workspace_root: &Path,
    config: &PipelineRunConfig,
    root_node_id: &str,
    root_screenshot_ref: &str,
) -> Result<(), PipelineError> {
    let screenshot_path = workspace_root.join(root_screenshot_ref);
    if screenshot_path.is_file() {
        return Ok(());
    }

    let live_config = match &config.fetch_mode {
        FetchMode::Live(config) => config,
        _ => return Ok(()),
    };

    let request = figma_client::LiveScreenshotRequest::new(
        live_config.file_key.clone(),
        root_node_id.to_string(),
        live_config.figma_token.clone(),
        live_config.api_base_url.clone(),
    )
    .map_err(fetch_client_error)?;
    let screenshot =
        figma_client::fetch_node_screenshot_live(&request).map_err(fetch_client_error)?;

    let response = reqwest::blocking::Client::new()
        .get(screenshot.image_url.as_str())
        .send()
        .map_err(|err| {
            PipelineError::FetchClient(format!("screenshot download transport error: {err}"))
        })?;
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "response body unavailable".to_string());
        return Err(PipelineError::FetchClient(format!(
            "screenshot download returned non-success status {}: {body}",
            status.as_u16()
        )));
    }
    let bytes = response.bytes().map_err(|err| {
        PipelineError::FetchClient(format!("screenshot download decode error: {err}"))
    })?;

    write_bytes(screenshot_path.as_path(), bytes.as_ref())
}

fn build_skeleton_nodes(root: &ui_spec::UiSpec) -> Vec<agent_context::SkeletonNode> {
    let mut nodes = Vec::new();
    flatten_skeleton(root, "", &mut nodes);
    nodes
}

fn flatten_skeleton(
    node: &ui_spec::UiSpec,
    parent_path: &str,
    out: &mut Vec<agent_context::SkeletonNode>,
) {
    let name = node_name(node).to_string();
    let path = if parent_path.is_empty() {
        name.clone()
    } else {
        format!("{parent_path}/{name}")
    };

    out.push(agent_context::SkeletonNode {
        node_id: node.id().to_string(),
        node_type: node_type_label(node).to_string(),
        name: name.clone(),
        path: path.clone(),
        children: node
            .children()
            .iter()
            .map(|child| child.id().to_string())
            .collect(),
    });

    for child in node.children() {
        flatten_skeleton(child, path.as_str(), out);
    }
}

fn build_search_index_entries(root: &ui_spec::UiSpec) -> Vec<agent_context::SearchIndexEntry> {
    let mut entries = Vec::new();
    flatten_search_entries(root, "", &mut entries);
    entries
}

fn flatten_search_entries(
    node: &ui_spec::UiSpec,
    parent_path: &str,
    out: &mut Vec<agent_context::SearchIndexEntry>,
) {
    let name = node_name(node).to_string();
    let path = if parent_path.is_empty() {
        name.clone()
    } else {
        format!("{parent_path}/{name}")
    };

    let mut raw_tokens = vec![name.clone()];
    if let ui_spec::UiSpec::Container { text, .. } = node
        && !text.is_empty()
    {
        raw_tokens.push(text.clone());
    }

    let mut normalized_tokens = std::collections::BTreeSet::new();
    for token in raw_tokens
        .iter()
        .flat_map(|value| agent_context::normalize_tokens(value))
    {
        normalized_tokens.insert(token);
    }

    let geometry_tags = infer_geometry_tags(path.as_str());

    out.push(agent_context::SearchIndexEntry {
        node_id: node.id().to_string(),
        name: name.clone(),
        node_type: node_type_label(node).to_string(),
        path: path.clone(),
        raw_tokens,
        normalized_tokens: normalized_tokens.into_iter().collect(),
        aliases: Vec::new(),
        geometry_tags,
    });

    for child in node.children() {
        flatten_search_entries(child, path.as_str(), out);
    }
}

fn infer_geometry_tags(path: &str) -> Vec<String> {
    let path_tokens = agent_context::normalize_tokens(path);
    let mut tags = std::collections::BTreeSet::new();
    for token in path_tokens {
        if matches!(
            token.as_str(),
            "header" | "footer" | "sidebar" | "left" | "right" | "center" | "body" | "content"
        ) {
            tags.insert(token);
        }
    }
    tags.into_iter().collect()
}

fn node_name(node: &ui_spec::UiSpec) -> &str {
    match node {
        ui_spec::UiSpec::Container { name, .. }
        | ui_spec::UiSpec::Instance { name, .. }
        | ui_spec::UiSpec::Text { name, .. }
        | ui_spec::UiSpec::Image { name, .. }
        | ui_spec::UiSpec::Shape { name, .. }
        | ui_spec::UiSpec::Vector { name, .. }
        | ui_spec::UiSpec::Button { name, .. }
        | ui_spec::UiSpec::ScrollView { name, .. }
        | ui_spec::UiSpec::HStack { name, .. }
        | ui_spec::UiSpec::VStack { name, .. }
        | ui_spec::UiSpec::ZStack { name, .. } => name.as_str(),
    }
}

fn node_type_label(node: &ui_spec::UiSpec) -> &'static str {
    match node.node_type() {
        ui_spec::NodeType::Container => "CONTAINER",
        ui_spec::NodeType::Instance => "INSTANCE",
        ui_spec::NodeType::Text => "TEXT",
        ui_spec::NodeType::Image => "IMAGE",
        ui_spec::NodeType::Shape => "SHAPE",
        ui_spec::NodeType::Vector => "VECTOR",
        ui_spec::NodeType::Button => "BUTTON",
        ui_spec::NodeType::ScrollView => "SCROLL_VIEW",
        ui_spec::NodeType::HStack => "HSTACK",
        ui_spec::NodeType::VStack => "VSTACK",
        ui_spec::NodeType::ZStack => "ZSTACK",
    }
}

fn map_search_status(status: agent_context::SearchStatus) -> FindNodesStatus {
    match status {
        agent_context::SearchStatus::Ok => FindNodesStatus::Ok,
        agent_context::SearchStatus::LowConfidence => FindNodesStatus::LowConfidence,
        agent_context::SearchStatus::NoMatch => FindNodesStatus::NoMatch,
        agent_context::SearchStatus::Ambiguous => FindNodesStatus::Ambiguous,
    }
}

fn find_nodes_status_label(status: &FindNodesStatus) -> &'static str {
    match status {
        FindNodesStatus::Ok => "ok",
        FindNodesStatus::LowConfidence => "low_confidence",
        FindNodesStatus::NoMatch => "no_match",
        FindNodesStatus::Ambiguous => "ambiguous",
    }
}

fn node_info_status_label(status: &NodeInfoStatus) -> &'static str {
    match status {
        NodeInfoStatus::Ok => "ok",
        NodeInfoStatus::NotFound => "not_found",
    }
}

fn ensure_generation_reports_exist(workspace_root: &Path) -> Result<(), PipelineError> {
    let warnings_path = workspace_root.join(GENERATION_WARNINGS_ARTIFACT_RELATIVE_PATH);
    if !warnings_path.exists() {
        let warnings = agent_context::GenerationWarnings {
            version: "generation_warnings/1.0".to_string(),
            warnings: Vec::new(),
        };
        let encoded = serde_json::to_vec_pretty(&warnings).map_err(serialization_error)?;
        write_bytes(warnings_path.as_path(), encoded.as_slice())?;
    }

    let trace_path = workspace_root.join(GENERATION_TRACE_ARTIFACT_RELATIVE_PATH);
    if !trace_path.exists() {
        let trace = agent_context::GenerationTrace {
            version: "generation_trace/1.0".to_string(),
            events: Vec::new(),
        };
        let encoded = serde_json::to_vec_pretty(&trace).map_err(serialization_error)?;
        write_bytes(trace_path.as_path(), encoded.as_slice())?;
    }

    Ok(())
}

fn append_warning(
    workspace_root: &Path,
    warning_type: &str,
    node_query: &str,
    candidate_node_ids: Vec<String>,
    agent_action: &str,
    message: &str,
) -> Result<(), PipelineError> {
    let warnings_path = workspace_root.join(GENERATION_WARNINGS_ARTIFACT_RELATIVE_PATH);
    let mut warnings = if warnings_path.exists() {
        let bytes = std::fs::read(warnings_path.as_path()).map_err(io_error)?;
        serde_json::from_slice::<agent_context::GenerationWarnings>(bytes.as_slice())
            .map_err(serialization_error)?
    } else {
        agent_context::GenerationWarnings {
            version: "generation_warnings/1.0".to_string(),
            warnings: Vec::new(),
        }
    };

    let next_id = format!("warning-{}", warnings.warnings.len() + 1);
    warnings.warnings.push(agent_context::GenerationWarning {
        warning_id: next_id,
        warning_type: warning_type.to_string(),
        severity: "warning".to_string(),
        node_query: node_query.to_string(),
        candidate_node_ids,
        agent_action: agent_action.to_string(),
        message: message.to_string(),
    });

    let encoded = serde_json::to_vec_pretty(&warnings).map_err(serialization_error)?;
    write_bytes(warnings_path.as_path(), encoded.as_slice())
}

fn append_trace_event(
    workspace_root: &Path,
    tool_name: &str,
    status: &str,
    query: &str,
    selected_node_ids: Vec<String>,
) -> Result<(), PipelineError> {
    let trace_path = workspace_root.join(GENERATION_TRACE_ARTIFACT_RELATIVE_PATH);
    let mut trace = if trace_path.exists() {
        let bytes = std::fs::read(trace_path.as_path()).map_err(io_error)?;
        serde_json::from_slice::<agent_context::GenerationTrace>(bytes.as_slice())
            .map_err(serialization_error)?
    } else {
        agent_context::GenerationTrace {
            version: "generation_trace/1.0".to_string(),
            events: Vec::new(),
        }
    };

    let next_id = format!("event-{}", trace.events.len() + 1);
    trace.events.push(agent_context::TraceEvent {
        event_id: next_id,
        tool_name: tool_name.to_string(),
        status: status.to_string(),
        query: query.to_string(),
        selected_node_ids,
    });

    let encoded = serde_json::to_vec_pretty(&trace).map_err(serialization_error)?;
    write_bytes(trace_path.as_path(), encoded.as_slice())
}

fn run_export_assets_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<crate::figma_client::normalizer::NormalizationOutput>(
        workspace_root,
        NORMALIZED_ARTIFACT_RELATIVE_PATH,
    )?;

    let assets = asset_pipeline::build_asset_manifest(&normalized);
    let encoded = serde_json::to_vec_pretty(&assets).map_err(serialization_error)?;

    let output_path = workspace_root.join(ASSET_MANIFEST_RELATIVE_PATH);
    write_bytes(output_path.as_path(), encoded.as_slice())?;

    Ok(normalize_result_path(workspace_root, output_path.as_path()))
}

fn build_artifact_ref(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<BundleArtifactRef, PipelineError> {
    let path = workspace_root.join(relative_path);
    if !path.exists() {
        return Err(PipelineError::MissingInputArtifact(
            relative_path.to_string(),
        ));
    }

    let bytes = std::fs::read(path.as_path()).map_err(io_error)?;
    Ok(BundleArtifactRef {
        path: relative_path.to_string(),
        sha256: hash::sha256_hex(bytes.as_slice()),
    })
}

fn build_optional_artifact_ref(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<Option<BundleArtifactRef>, PipelineError> {
    if relative_path.trim().is_empty() {
        return Ok(None);
    }

    let path = workspace_root.join(relative_path);
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(path.as_path()).map_err(io_error)?;
    Ok(Some(BundleArtifactRef {
        path: relative_path.to_string(),
        sha256: hash::sha256_hex(bytes.as_slice()),
    }))
}

fn build_bundle_instructions(workspace_root: &Path) -> Result<BundleInstructions, PipelineError> {
    let skills_guide_path = ".codex/SKILLS.md";
    let skills_guide_markdown = read_required_text_file(workspace_root, skills_guide_path)?;
    let agent_playbook_markdown =
        read_required_text_file(workspace_root, "docs/agent-playbook.md")?;
    let figma_ui_coder_markdown =
        read_required_text_file(workspace_root, "docs/figma-ui-coder.md")?;

    let mut skill_docs = Vec::new();
    for (name, path) in parse_active_skill_refs(skills_guide_markdown.as_str()) {
        let markdown = read_required_text_file(workspace_root, path.as_str())?;
        skill_docs.push(BundleSkillDoc {
            name,
            path,
            markdown,
        });
    }

    Ok(BundleInstructions {
        skills_guide_markdown,
        agent_playbook_markdown,
        figma_ui_coder_markdown,
        skill_docs,
    })
}

fn parse_active_skill_refs(skills_guide_markdown: &str) -> Vec<(String, String)> {
    let mut in_active_skills = false;
    let mut current_name: Option<String> = None;
    let mut refs = Vec::new();
    let mut seen_paths = std::collections::BTreeSet::new();

    for raw_line in skills_guide_markdown.lines() {
        let line = raw_line.trim();
        if line == "## Active Skills" {
            in_active_skills = true;
            continue;
        }
        if in_active_skills && line.starts_with("## ") {
            break;
        }
        if !in_active_skills {
            continue;
        }

        if let Some(path) = line.strip_prefix("Path:")
            && let Some(name) = current_name.take()
        {
            let normalized_path = path.trim().trim_matches('`').to_string();
            if seen_paths.insert(normalized_path.clone()) {
                refs.push((name, normalized_path));
            }
            continue;
        }

        if let Some(name) = extract_backticked_name(line) {
            current_name = Some(name);
        }
    }

    refs
}

fn extract_backticked_name(line: &str) -> Option<String> {
    let start = line.find('`')?;
    let tail = &line[start + 1..];
    let end = tail.find('`')?;
    if end == 0 {
        return None;
    }
    Some(tail[..end].to_string())
}

fn read_required_text_file(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<String, PipelineError> {
    let path = workspace_root.join(relative_path);
    if !path.exists() {
        return Err(PipelineError::MissingInputArtifact(
            relative_path.to_string(),
        ));
    }

    std::fs::read_to_string(path.as_path()).map_err(io_error)
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

fn read_required_ron<T>(workspace_root: &Path, relative_path: &str) -> Result<T, PipelineError>
where
    T: serde::de::DeserializeOwned,
{
    let path = workspace_root.join(relative_path);
    if !path.exists() {
        return Err(PipelineError::MissingInputArtifact(
            relative_path.to_string(),
        ));
    }

    let text = std::fs::read_to_string(path.as_path()).map_err(io_error)?;
    ron::de::from_str(text.as_str()).map_err(serialization_error)
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

fn normalizer_error(err: crate::figma_client::normalizer::NormalizationError) -> PipelineError {
    PipelineError::Normalizer(err.to_string())
}

fn ui_spec_build_error(err: ui_spec::UiSpecBuildError) -> PipelineError {
    PipelineError::UiSpecBuild(err.to_string())
}

#[cfg(test)]
mod tests;

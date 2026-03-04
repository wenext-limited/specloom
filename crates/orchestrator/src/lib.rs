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
        "build-agent-context" => Some(run_build_agent_context_stage(workspace_root)?),
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

fn run_build_spec_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let normalized = read_required_json::<figma_normalizer::NormalizationOutput>(
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

    let transform_plan = generate_transform_plan(&pre_layout, &node_map)?;
    let transform_plan_path = workspace_root.join(TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH);
    let transform_plan_bytes =
        serde_json::to_vec_pretty(&transform_plan).map_err(serialization_error)?;
    write_bytes(transform_plan_path.as_path(), transform_plan_bytes.as_slice())?;

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
    normalized: &figma_normalizer::NormalizationOutput,
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
    _pre_layout: &ui_spec::UiSpec,
    _node_map: &NodeMapArtifact,
) -> Result<ui_spec::TransformPlan, PipelineError> {
    Ok(ui_spec::TransformPlan::default())
}

fn run_build_agent_context_stage(workspace_root: &Path) -> Result<String, PipelineError> {
    let spec = read_required_ron::<ui_spec::UiSpec>(workspace_root, SPEC_ARTIFACT_RELATIVE_PATH)?;

    let root_node_id = spec.id().to_string();
    let context = agent_context::AgentContext {
        version: "agent_context/1.0".to_string(),
        screen: agent_context::ScreenRef {
            root_node_id: root_node_id.clone(),
            root_screenshot_ref: format!(
                "output/images/root_{}.png",
                root_node_id.replace(':', "_")
            ),
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

    Ok(normalize_result_path(workspace_root, context_path.as_path()))
}

fn build_skeleton_nodes(root: &ui_spec::UiSpec) -> Vec<agent_context::SkeletonNode> {
    let mut nodes = Vec::new();
    flatten_skeleton(root, "", &mut nodes);
    nodes
}

fn flatten_skeleton(node: &ui_spec::UiSpec, parent_path: &str, out: &mut Vec<agent_context::SkeletonNode>) {
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
                "build-spec",
                "build-agent-context",
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
                ("build-spec", "output/specs"),
                ("build-agent-context", "output/agent"),
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

        let pre_layout_path = workspace_root.join("output/specs/pre_layout.ron");
        assert!(pre_layout_path.is_file());
        let pre_layout =
            std::fs::read_to_string(pre_layout_path).expect("pre-layout should be readable");
        assert!(pre_layout.contains("Container("));

        let node_map_path = workspace_root.join("output/specs/node_map.json");
        assert!(node_map_path.is_file());
        let node_map = std::fs::read_to_string(node_map_path).expect("node map should be readable");
        let node_map_value: serde_json::Value =
            serde_json::from_str(node_map.as_str()).expect("node map should decode");
        assert_eq!(node_map_value["version"], "node_map/1.0");
        assert!(node_map_value["nodes"].is_object());

        let transform_plan_path = workspace_root.join("output/specs/transform_plan.json");
        assert!(transform_plan_path.is_file());
        let transform_plan = std::fs::read_to_string(transform_plan_path)
            .expect("transform plan should be readable");
        let transform_plan_value: serde_json::Value =
            serde_json::from_str(transform_plan.as_str()).expect("transform plan should decode");
        assert_eq!(transform_plan_value["version"], "transform_plan/1.0");
        assert!(transform_plan_value["decisions"].is_array());

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
    fn run_stage_build_agent_context_writes_agent_artifacts() {
        let workspace_root =
            unique_test_workspace_root("run_stage_build_agent_context_writes_agent_artifacts");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run third");

        let result = run_stage_in_workspace("build-agent-context", workspace_root.as_path())
            .expect("build-agent-context should run");
        assert_eq!(
            result,
            StageExecutionResult {
                stage_name: "build-agent-context",
                output_dir: "output/agent",
                artifact_path: Some("output/agent/agent_context.json".to_string()),
            }
        );

        assert!(workspace_root.join("output/agent/agent_context.json").is_file());
        assert!(workspace_root.join("output/agent/search_index.json").is_file());

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn find_nodes_in_workspace_returns_ranked_candidates() {
        let workspace_root =
            unique_test_workspace_root("find_nodes_in_workspace_returns_ranked_candidates");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run third");
        run_stage_in_workspace("build-agent-context", workspace_root.as_path())
            .expect("build-agent-context should run fourth");

        let result = find_nodes_in_workspace(workspace_root.as_path(), "fixture root", 5)
            .expect("find_nodes should succeed");
        assert_eq!(result.status, FindNodesStatus::LowConfidence);
        assert!(!result.candidates.is_empty());
        assert_eq!(result.candidates[0].node_id, "0:1");

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn get_node_info_in_workspace_returns_not_found_for_missing_node() {
        let workspace_root =
            unique_test_workspace_root("get_node_info_in_workspace_returns_not_found");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run third");
        run_stage_in_workspace("build-agent-context", workspace_root.as_path())
            .expect("build-agent-context should run fourth");

        let result = get_node_info_in_workspace(workspace_root.as_path(), "missing")
            .expect("node info lookup should succeed");
        assert_eq!(result.status, NodeInfoStatus::NotFound);
        assert!(result.node.is_none());

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn tool_lookup_no_match_emits_warning_artifact_entry() {
        let workspace_root =
            unique_test_workspace_root("tool_lookup_no_match_emits_warning_artifact_entry");

        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
        run_stage_in_workspace("normalize", workspace_root.as_path())
            .expect("normalize should run second");
        run_stage_in_workspace("build-spec", workspace_root.as_path())
            .expect("build-spec should run third");
        run_stage_in_workspace("build-agent-context", workspace_root.as_path())
            .expect("build-agent-context should run fourth");

        let result = find_nodes_in_workspace(workspace_root.as_path(), "query-that-does-not-match", 5)
            .expect("find_nodes should succeed");
        assert_eq!(result.status, FindNodesStatus::NoMatch);

        let warnings_path = workspace_root.join("output/reports/generation_warnings.json");
        assert!(warnings_path.is_file());

        let warnings_json =
            std::fs::read_to_string(warnings_path).expect("warnings artifact should be readable");
        let warnings: agent_context::GenerationWarnings =
            serde_json::from_str(warnings_json.as_str()).expect("warnings artifact should decode");
        assert!(warnings.warnings.iter().any(|warning| warning.warning_type == "NODE_NOT_FOUND"));

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
                "build-spec",
                "build-agent-context",
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

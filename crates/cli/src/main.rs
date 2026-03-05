#![forbid(unsafe_code)]

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "specloom")]
#[command(about = "Figma node tree to spec-first pipeline CLI")]
#[command(
    long_about = "Run deterministic pipeline stages for Figma snapshot processing, spec building, and agent lookup context."
)]
#[command(
    after_long_help = "Examples:\n  specloom generate --input fixture\n  specloom generate --input live --figma-url \"https://www.figma.com/design/<FILE_KEY>/<PAGE>?node-id=<NODE_ID>\"\n  specloom run-stage build-spec\n  specloom agent-tool find-nodes --query \"login button\" --output json"
)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Fetch raw snapshot data into `output/raw/fetch_snapshot.json`.
    #[command(
        long_about = "Fetch the source Figma snapshot. Use fixture mode for deterministic local runs, live mode for direct Figma API fetch, or snapshot mode to replay an existing raw artifact."
    )]
    Fetch {
        #[command(flatten)]
        input: FetchInputOptions,
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Normalize raw snapshot into `output/normalized/normalized_document.json`.
    Normalize,
    /// Build specs under `output/specs` (`pre_layout`, `node_map`, `transform_plan`, `ui_spec`).
    BuildSpec,
    /// Export image/vector asset manifest to `output/assets/asset_manifest.json`.
    ExportAssets,
    /// Run the full deterministic pipeline in stage order.
    #[command(
        long_about = "Run all default stages in order:\n  fetch -> normalize -> build-spec -> build-agent-context -> export-assets"
    )]
    Generate {
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
        #[command(flatten)]
        input: GenerateInputOptions,
    },
    /// List available stages and their output directories.
    Stages {
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Run one stage by name (see `specloom stages`).
    RunStage {
        /// Stage name (fetch, normalize, build-spec, build-agent-context, export-assets).
        #[arg(value_name = "STAGE")]
        stage: String,
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Run stateless agent helper tools (`find-nodes`, `get-node-info`, `get-node-screenshot`).
    AgentTool {
        #[command(subcommand)]
        tool: AgentToolCommand,
    },
}

#[derive(Debug, clap::Subcommand)]
enum AgentToolCommand {
    /// Search indexed nodes with deterministic fuzzy ranking.
    FindNodes {
        /// Free-text query from UI context (labels, structure hints, etc.).
        #[arg(long)]
        query: String,
        /// Max number of ranked candidates to return.
        #[arg(long, default_value_t = 5)]
        top_k: usize,
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Read indexed metadata for one node id.
    GetNodeInfo {
        /// Node id to inspect (for example `79:18523`).
        #[arg(long)]
        node_id: String,
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Fetch screenshot URL for one node via Figma images API.
    #[command(
        long_about = "Fetch a single-node screenshot from Figma and return its URL. Requires `--file-key`, `--node-id`, and FIGMA_TOKEN (or `--figma-token`)."
    )]
    GetNodeScreenshot {
        /// Figma file key.
        #[arg(long)]
        file_key: String,
        /// Figma node id (`:` or `-` format accepted).
        #[arg(long)]
        node_id: String,
        /// Figma personal access token (falls back to `FIGMA_TOKEN` env var).
        #[arg(long)]
        figma_token: Option<String>,
        #[arg(long, hide = true)]
        figma_api_base_url: Option<String>,
        /// Output format for command results.
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputMode {
    Text,
    Json,
}

#[derive(Debug, Clone, clap::Args)]
struct FetchInputOptions {
    /// Input mode: `fixture`, `live`, or `snapshot`.
    #[arg(long, value_enum, default_value_t = InputMode::Fixture)]
    input: InputMode,
    /// Path to an existing raw snapshot JSON (required for `--input snapshot`).
    #[arg(long)]
    snapshot_path: Option<String>,
    /// Figma quick link containing file key + node-id (used with `--input live`).
    #[arg(long)]
    figma_url: Option<String>,
    /// Figma file key (used with `--input live`, optional when `--figma-url` is provided).
    #[arg(long)]
    file_key: Option<String>,
    /// Figma node id (used with `--input live`, optional when `--figma-url` is provided).
    #[arg(long)]
    node_id: Option<String>,
    /// Figma personal access token (falls back to `FIGMA_TOKEN` env var).
    #[arg(long)]
    figma_token: Option<String>,
    #[arg(long, hide = true)]
    figma_api_base_url: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
struct GenerateInputOptions {
    /// Input mode for `generate`: defaults to `live` (use `--input fixture` for local deterministic runs).
    #[arg(long, value_enum, default_value_t = InputMode::Live)]
    input: InputMode,
    /// Path to an existing raw snapshot JSON (required for `--input snapshot`).
    #[arg(long)]
    snapshot_path: Option<String>,
    /// Figma quick link containing file key + node-id (used with `--input live`).
    #[arg(long)]
    figma_url: Option<String>,
    /// Figma file key (used with `--input live`, optional when `--figma-url` is provided).
    #[arg(long)]
    file_key: Option<String>,
    /// Figma node id (used with `--input live`, optional when `--figma-url` is provided).
    #[arg(long)]
    node_id: Option<String>,
    /// Figma personal access token (falls back to `FIGMA_TOKEN` env var).
    #[arg(long)]
    figma_token: Option<String>,
    #[arg(long, hide = true)]
    figma_api_base_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
enum InputMode {
    #[default]
    Fixture,
    Live,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedFigmaQuickLink {
    file_key: String,
    node_id: String,
}

fn main() {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Command::Fetch { input, output } => {
                let config = fetch_config_or_exit(&input, output);
                match core::run_stage_with_config("fetch", &config) {
                    Ok(result) => match output {
                        OutputMode::Text => {
                            println!("stage={} output={}", result.stage_name, result.output_dir);
                        }
                        OutputMode::Json => {
                            println!(
                                "{{\"stage\":\"{}\",\"output\":\"{}\"}}",
                                json_escape(result.stage_name),
                                json_escape(result.output_dir)
                            );
                        }
                    },
                    Err(err) => emit_error_and_exit(err, output),
                }
            }
            Command::Stages { output } => {
                let stages = core::pipeline_stage_output_dirs();
                match output {
                    OutputMode::Text => {
                        for (stage, output) in stages {
                            println!("stage={stage} output={output}");
                        }
                    }
                    OutputMode::Json => {
                        let stages = stages
                            .into_iter()
                            .map(|(stage, output)| {
                                format!(
                                    "{{\"stage\":\"{}\",\"output\":\"{}\"}}",
                                    json_escape(stage),
                                    json_escape(output)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        println!("{{\"stages\":[{stages}]}}");
                    }
                }
            }
            Command::RunStage { stage, output } => match core::run_stage(&stage) {
                Ok(result) => match output {
                    OutputMode::Text => {
                        println!("stage={} output={}", result.stage_name, result.output_dir);
                    }
                    OutputMode::Json => {
                        println!(
                            "{{\"stage\":\"{}\",\"output\":\"{}\"}}",
                            json_escape(result.stage_name),
                            json_escape(result.output_dir)
                        );
                    }
                },
                Err(err) => emit_error_and_exit(err, output),
            },
            Command::Generate { output, input } => {
                let config = fetch_config_for_generate_or_exit(&input, output);
                match output {
                    OutputMode::Text => {
                        let stage_names = core::pipeline_stage_names();
                        let total_stages = stage_names.len();
                        println!(
                            "pipeline=generate mode={} stages={total_stages}",
                            input_mode_label(input.input)
                        );
                        let mut results = Vec::with_capacity(total_stages);

                        for (index, stage_name) in stage_names.into_iter().enumerate() {
                            let position = index + 1;
                            println!("[{position}/{total_stages}] RUN  stage={stage_name}");
                            match core::run_stage_with_config(stage_name, &config) {
                                Ok(result) => {
                                    println!(
                                        "[{position}/{total_stages}] DONE stage={} output={}",
                                        result.stage_name, result.output_dir
                                    );
                                    results.push(result);
                                }
                                Err(err) => {
                                    println!("[{position}/{total_stages}] FAIL stage={stage_name}");
                                    emit_error_and_exit(err, output);
                                }
                            }
                        }

                        println!("summary=ok completed={total_stages}/{total_stages} failed=0");
                        for result in results {
                            if let Some(artifact_path) = result.artifact_path {
                                println!(
                                    "stage={} output={} artifact={}",
                                    result.stage_name, result.output_dir, artifact_path
                                );
                            } else {
                                println!(
                                    "stage={} output={}",
                                    result.stage_name, result.output_dir
                                );
                            }
                        }
                    }
                    OutputMode::Json => match core::run_all_with_config(&config) {
                        Ok(results) => {
                            let results = results
                                .into_iter()
                                .map(|result| {
                                    let artifact = if let Some(path) = result.artifact_path {
                                        format!("\"{}\"", json_escape(path.as_str()))
                                    } else {
                                        "null".to_string()
                                    };
                                    format!(
                                        "{{\"stage\":\"{}\",\"output\":\"{}\",\"artifact\":{}}}",
                                        json_escape(result.stage_name),
                                        json_escape(result.output_dir),
                                        artifact
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            println!("{{\"results\":[{results}]}}");
                        }
                        Err(err) => emit_error_and_exit(err, output),
                    },
                }
            }
            Command::AgentTool { tool } => match tool {
                AgentToolCommand::FindNodes {
                    query,
                    top_k,
                    output,
                } => match core::find_nodes(query.as_str(), top_k) {
                    Ok(result) => match output {
                        OutputMode::Text => {
                            println!(
                                "status={} query={} candidates={}",
                                find_nodes_status_label(&result.status),
                                query,
                                result.candidates.len()
                            );
                            for candidate in result.candidates {
                                let reasons = candidate.match_reasons.join(",");
                                println!(
                                    "node_id={} score={:.3} reasons={}",
                                    candidate.node_id, candidate.score, reasons
                                );
                            }
                        }
                        OutputMode::Json => {
                            let encoded = serde_json::to_string(&result).unwrap_or_else(|err| {
                                panic!("find-nodes json encode failed: {err}")
                            });
                            println!("{encoded}");
                        }
                    },
                    Err(err) => emit_error_and_exit(err, output),
                },
                AgentToolCommand::GetNodeInfo { node_id, output } => {
                    match core::get_node_info(node_id.as_str()) {
                        Ok(result) => match output {
                            OutputMode::Text => {
                                if let Some(node) = result.node {
                                    println!(
                                        "status={} node_id={} name={} type={} path={}",
                                        node_info_status_label(&result.status),
                                        node.node_id,
                                        node.name,
                                        node.node_type,
                                        node.path
                                    );
                                } else {
                                    println!(
                                        "status={} node_id={}",
                                        node_info_status_label(&result.status),
                                        node_id
                                    );
                                }
                            }
                            OutputMode::Json => {
                                let encoded =
                                    serde_json::to_string(&result).unwrap_or_else(|err| {
                                        panic!("get-node-info json encode failed: {err}")
                                    });
                                println!("{encoded}");
                            }
                        },
                        Err(err) => emit_error_and_exit(err, output),
                    }
                }
                AgentToolCommand::GetNodeScreenshot {
                    file_key,
                    node_id,
                    figma_token,
                    figma_api_base_url,
                    output,
                } => {
                    let figma_token = normalize_optional_field(figma_token.as_deref())
                        .or_else(figma_token_from_env)
                        .unwrap_or_else(|| {
                            emit_usage_error_and_exit(
                                "get-node-screenshot missing required value(s): FIGMA_TOKEN (or --figma-token). Provide the missing value(s) and retry.",
                                output,
                            )
                        });
                    let request = core::figma_client::LiveScreenshotRequest::new(
                        file_key,
                        node_id,
                        figma_token,
                        normalize_optional_field(figma_api_base_url.as_deref()),
                    )
                    .unwrap_or_else(|err| emit_fetch_error_and_exit(err, output));

                    match core::figma_client::fetch_node_screenshot_live(&request) {
                        Ok(result) => match output {
                            OutputMode::Text => {
                                println!(
                                    "status=ok node_id={} format={} image_url={}",
                                    result.node_id, result.format, result.image_url
                                );
                            }
                            OutputMode::Json => {
                                let encoded =
                                    serde_json::to_string(&result).unwrap_or_else(|err| {
                                        panic!("get-node-screenshot json encode failed: {err}")
                                    });
                                println!("{encoded}");
                            }
                        },
                        Err(err) => emit_fetch_error_and_exit(err, output),
                    }
                }
            },
            _ => {
                let stage_name = command.stage_name();
                if let Some((_, output_dir)) = core::pipeline_stage_output_dirs()
                    .into_iter()
                    .find(|(name, _)| *name == stage_name)
                {
                    println!("stage={stage_name} output={output_dir}");
                }
            }
        }
    }
}

impl Command {
    fn stage_name(&self) -> &'static str {
        match self {
            Command::Fetch { .. } => "fetch",
            Command::Normalize => "normalize",
            Command::BuildSpec => "build-spec",
            Command::ExportAssets => "export-assets",
            Command::Generate { .. } => "generate",
            Command::Stages { .. } => "stages",
            Command::RunStage { .. } => "run-stage",
            Command::AgentTool { .. } => "agent-tool",
        }
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn emit_error_and_exit(error: core::PipelineError, output: OutputMode) -> ! {
    let message = error.actionable_message();
    match output {
        OutputMode::Text => eprintln!("{message}"),
        OutputMode::Json => {
            eprintln!("{{\"error\":\"{}\"}}", json_escape(&message));
        }
    }
    std::process::exit(2);
}

fn emit_fetch_error_and_exit(error: core::figma_client::FetchClientError, output: OutputMode) -> ! {
    let message = error.to_string();
    match output {
        OutputMode::Text => eprintln!("{message}"),
        OutputMode::Json => eprintln!("{{\"error\":\"{}\"}}", json_escape(&message)),
    }
    std::process::exit(2);
}

fn find_nodes_status_label(status: &core::FindNodesStatus) -> &'static str {
    match status {
        core::FindNodesStatus::Ok => "ok",
        core::FindNodesStatus::LowConfidence => "low_confidence",
        core::FindNodesStatus::NoMatch => "no_match",
        core::FindNodesStatus::Ambiguous => "ambiguous",
    }
}

fn node_info_status_label(status: &core::NodeInfoStatus) -> &'static str {
    match status {
        core::NodeInfoStatus::Ok => "ok",
        core::NodeInfoStatus::NotFound => "not_found",
    }
}

fn fetch_config_or_exit(input: &FetchInputOptions, output: OutputMode) -> core::PipelineRunConfig {
    build_fetch_config(input).unwrap_or_else(|message| emit_usage_error_and_exit(&message, output))
}

fn fetch_config_for_generate_or_exit(
    input: &GenerateInputOptions,
    output: OutputMode,
) -> core::PipelineRunConfig {
    let fetch_input = FetchInputOptions {
        input: input.input,
        snapshot_path: input.snapshot_path.clone(),
        figma_url: input.figma_url.clone(),
        file_key: input.file_key.clone(),
        node_id: input.node_id.clone(),
        figma_token: input.figma_token.clone(),
        figma_api_base_url: input.figma_api_base_url.clone(),
    };
    build_fetch_config(&fetch_input)
        .unwrap_or_else(|message| emit_usage_error_and_exit(&message, output))
}

fn build_fetch_config(input: &FetchInputOptions) -> Result<core::PipelineRunConfig, String> {
    match input.input {
        InputMode::Fixture => Ok(core::PipelineRunConfig::default()),
        InputMode::Live => {
            let parsed_quick_link = normalize_optional_field(input.figma_url.as_deref())
                .map(|url| parse_figma_quick_link(url.as_str()))
                .transpose()?;
            let file_key = normalize_optional_field(input.file_key.as_deref()).or_else(|| {
                parsed_quick_link
                    .as_ref()
                    .map(|parsed| parsed.file_key.clone())
            });
            let node_id = normalize_optional_field(input.node_id.as_deref()).or_else(|| {
                parsed_quick_link
                    .as_ref()
                    .map(|parsed| parsed.node_id.clone())
            });
            let figma_token = normalize_optional_field(input.figma_token.as_deref())
                .or_else(figma_token_from_env);
            let api_base_url = normalize_optional_field(input.figma_api_base_url.as_deref());

            let mut missing_values = Vec::new();
            if file_key.is_none() {
                missing_values.push("--file-key (or --figma-url)");
            }
            if node_id.is_none() {
                missing_values.push("--node-id (or --figma-url)");
            }
            if figma_token.is_none() {
                missing_values.push("FIGMA_TOKEN (or --figma-token)");
            }

            if !missing_values.is_empty() {
                return Err(format!(
                    "live input missing required value(s): {}. Provide the missing value(s) and retry.",
                    missing_values.join(", ")
                ));
            }

            Ok(core::PipelineRunConfig {
                fetch_mode: core::FetchMode::Live(core::LiveFetchConfig {
                    file_key: file_key.expect("checked above"),
                    node_id: node_id.expect("checked above"),
                    figma_token: figma_token.expect("checked above"),
                    api_base_url,
                }),
            })
        }
        InputMode::Snapshot => {
            let snapshot_path = normalize_optional_field(input.snapshot_path.as_deref());
            if snapshot_path.is_none() {
                return Err(
                    "snapshot input missing required value(s): --snapshot-path. Provide the missing value(s) and retry."
                        .to_string(),
                );
            }
            Ok(core::PipelineRunConfig {
                fetch_mode: core::FetchMode::Snapshot(core::SnapshotFetchConfig {
                    snapshot_path: snapshot_path.expect("checked above"),
                }),
            })
        }
    }
}

fn figma_token_from_env() -> Option<String> {
    std::env::var("FIGMA_TOKEN")
        .ok()
        .and_then(|value| normalize_optional_field(Some(value.as_str())))
}

fn input_mode_label(mode: InputMode) -> &'static str {
    match mode {
        InputMode::Fixture => "fixture",
        InputMode::Live => "live",
        InputMode::Snapshot => "snapshot",
    }
}

fn normalize_optional_field(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn emit_usage_error_and_exit(message: &str, output: OutputMode) -> ! {
    match output {
        OutputMode::Text => eprintln!("{message}"),
        OutputMode::Json => eprintln!("{{\"error\":\"{}\"}}", json_escape(message)),
    }
    std::process::exit(2);
}

fn parse_figma_quick_link(value: &str) -> Result<ParsedFigmaQuickLink, String> {
    let parsed = url::Url::parse(value)
        .map_err(|err| format!("invalid --figma-url: {err}. Provide a valid Figma design URL."))?;

    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if host != "figma.com" && host != "www.figma.com" {
        return Err("invalid --figma-url: host must be figma.com or www.figma.com.".to_string());
    }

    let segments: Vec<_> = parsed
        .path_segments()
        .map(|parts| parts.collect())
        .unwrap_or_default();
    let file_key = match segments.as_slice() {
        [kind, key, ..] if matches!(*kind, "design" | "file" | "proto") && !key.is_empty() => {
            key.to_string()
        }
        _ => {
            return Err("invalid --figma-url: could not parse file key from URL path.".to_string());
        }
    };

    let node_id = parsed
        .query_pairs()
        .find(|(key, _)| key == "node-id")
        .map(|(_, value)| value.to_string())
        .map(|node_id| normalize_figma_node_id(node_id.as_str()))
        .filter(|node_id| !node_id.is_empty())
        .ok_or_else(|| {
            "invalid --figma-url: missing required node-id query parameter.".to_string()
        })?;

    Ok(ParsedFigmaQuickLink { file_key, node_id })
}

fn normalize_figma_node_id(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    if value.contains(':') {
        return value.to_string();
    }
    if let Some((left, right)) = value.split_once('-')
        && !left.is_empty()
        && !right.is_empty()
    {
        return format!("{left}:{right}");
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{ParsedFigmaQuickLink, normalize_figma_node_id, parse_figma_quick_link};

    #[test]
    fn parse_figma_quick_link_extracts_file_key_and_node_id() {
        let parsed = parse_figma_quick_link(
            "https://www.figma.com/design/iGk9NrpbnaoODjdoWc2P0g/Test?node-id=79-18523&m=dev",
        )
        .expect("quick link should parse");

        assert_eq!(
            parsed,
            ParsedFigmaQuickLink {
                file_key: "iGk9NrpbnaoODjdoWc2P0g".to_string(),
                node_id: "79:18523".to_string(),
            }
        );
    }

    #[test]
    fn parse_figma_quick_link_rejects_missing_node_id() {
        let err =
            parse_figma_quick_link("https://www.figma.com/design/iGk9NrpbnaoODjdoWc2P0g/Test")
                .expect_err("missing node-id should fail");
        assert!(err.contains("missing required node-id"));
    }

    #[test]
    fn normalize_figma_node_id_preserves_colon_format() {
        assert_eq!(normalize_figma_node_id("79:18523"), "79:18523");
    }
}

#![forbid(unsafe_code)]

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "cli")]
#[command(about = "Figma node tree to spec-first pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Fetch {
        #[command(flatten)]
        input: FetchInputOptions,
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    Normalize,
    InferLayout,
    BuildSpec,
    ExportAssets,
    Generate {
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
        #[command(flatten)]
        input: FetchInputOptions,
    },
    Stages {
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    RunStage {
        stage: String,
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
    #[arg(long, value_enum, default_value_t = InputMode::Fixture)]
    input: InputMode,
    #[arg(long)]
    snapshot_path: Option<String>,
    #[arg(long)]
    figma_url: Option<String>,
    #[arg(long)]
    file_key: Option<String>,
    #[arg(long)]
    node_id: Option<String>,
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
                match orchestrator::run_stage_with_config("fetch", &config) {
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
                let stages = orchestrator::pipeline_stage_output_dirs();
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
            Command::RunStage { stage, output } => match orchestrator::run_stage(&stage) {
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
                let config = fetch_config_or_exit(&input, output);
                match orchestrator::run_all_with_config(&config) {
                    Ok(results) => match output {
                        OutputMode::Text => {
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
                        OutputMode::Json => {
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
                    },
                    Err(err) => emit_error_and_exit(err, output),
                }
            }
            _ => {
                let stage_name = command.stage_name();
                if let Some((_, output_dir)) = orchestrator::pipeline_stage_output_dirs()
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
            Command::InferLayout => "infer-layout",
            Command::BuildSpec => "build-spec",
            Command::ExportAssets => "export-assets",
            Command::Generate { .. } => "generate",
            Command::Stages { .. } => "stages",
            Command::RunStage { .. } => "run-stage",
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

fn emit_error_and_exit(error: orchestrator::PipelineError, output: OutputMode) -> ! {
    let message = error.actionable_message();
    match output {
        OutputMode::Text => eprintln!("{message}"),
        OutputMode::Json => {
            eprintln!("{{\"error\":\"{}\"}}", json_escape(&message));
        }
    }
    std::process::exit(2);
}

fn fetch_config_or_exit(
    input: &FetchInputOptions,
    output: OutputMode,
) -> orchestrator::PipelineRunConfig {
    build_fetch_config(input).unwrap_or_else(|message| emit_usage_error_and_exit(&message, output))
}

fn build_fetch_config(
    input: &FetchInputOptions,
) -> Result<orchestrator::PipelineRunConfig, String> {
    match input.input {
        InputMode::Fixture => Ok(orchestrator::PipelineRunConfig::default()),
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

            Ok(orchestrator::PipelineRunConfig {
                fetch_mode: orchestrator::FetchMode::Live(orchestrator::LiveFetchConfig {
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
            Ok(orchestrator::PipelineRunConfig {
                fetch_mode: orchestrator::FetchMode::Snapshot(orchestrator::SnapshotFetchConfig {
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

#![forbid(unsafe_code)]

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "cli")]
#[command(about = "Figma node tree to SwiftUI pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Fetch,
    Normalize,
    InferLayout,
    BuildSpec,
    GenSwiftui,
    ExportAssets,
    Report,
    Generate {
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
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

fn main() {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
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
            Command::Generate { output } => match orchestrator::run_all() {
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
            },
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
            Command::Fetch => "fetch",
            Command::Normalize => "normalize",
            Command::InferLayout => "infer-layout",
            Command::BuildSpec => "build-spec",
            Command::GenSwiftui => "gen-swiftui",
            Command::ExportAssets => "export-assets",
            Command::Report => "report",
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

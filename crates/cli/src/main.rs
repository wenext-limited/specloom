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
    Stages,
}

fn main() {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Command::Stages => {
                for (stage, output) in orchestrator::pipeline_stage_output_dirs() {
                    println!("stage={stage} output={output}");
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
            Command::Fetch => "fetch",
            Command::Normalize => "normalize",
            Command::InferLayout => "infer-layout",
            Command::BuildSpec => "build-spec",
            Command::GenSwiftui => "gen-swiftui",
            Command::ExportAssets => "export-assets",
            Command::Report => "report",
            Command::Stages => "stages",
        }
    }
}

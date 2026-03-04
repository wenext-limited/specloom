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
}

fn main() {
    let _ = Cli::parse();
}

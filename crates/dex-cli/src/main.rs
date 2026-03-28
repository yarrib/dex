//! dex CLI — single-binary Rust CLI for data project operations.

mod commands;
mod output;

use clap::Parser;

/// Extensible CLI for data project operations.
#[derive(Parser)]
#[command(name = "dex", version, about)]
enum Cli {
    /// Scaffold a new project from a template.
    Init(commands::init::InitArgs),
    /// Agent project scaffolding for Databricks.
    Agent(commands::agent::AgentArgs),
    /// Run a pass-through command defined in dex.toml.
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli {
        Cli::Init(args) => commands::init::run(args),
        Cli::Agent(args) => commands::agent::run(args),
        Cli::External(args) => commands::passthrough::run(args),
    };

    if let Err(e) = result {
        output::print_error(&e.to_string());
        std::process::exit(1);
    }
}

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
    /// Scaffold and manage AI agent projects.
    Agent(commands::agent::AgentArgs),
    /// Add a composable trait to an existing project.
    Add(commands::add::AddArgs),
    /// Manage and install AI agent skill packs.
    Skills(commands::skills::SkillsArgs),
    /// MCP server for AI tool integration.
    Mcp(commands::mcp::McpArgs),
    /// List and inspect available templates.
    Templates(commands::templates::TemplatesArgs),
    /// Run a task defined in dex.toml.
    Run(commands::run::RunArgs),
    /// Build the project-memory knowledge graph in .context/wiki/.
    Context(commands::context::ContextArgs),
    /// Run a pass-through command defined in dex.toml.
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli {
        Cli::Init(args) => commands::init::run(args),
        Cli::Agent(args) => commands::agent::run(args),
        Cli::Add(args) => commands::add::run(args),
        Cli::Skills(args) => commands::skills::run(args),
        Cli::Mcp(args) => commands::mcp::run(args),
        Cli::Templates(args) => commands::templates::run(args),
        Cli::Run(args) => commands::run::run(args),
        Cli::Context(args) => commands::context::run(args),
        Cli::External(args) => commands::passthrough::run(args),
    };

    if let Err(e) = result {
        output::print_error(&e.to_string());
        std::process::exit(1);
    }
}

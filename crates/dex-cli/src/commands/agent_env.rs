//! `dex agent-env` — generate a reproducible dev environment for coding agents.
//!
//! Reads `dex.agent-env.toml` and generates a devcontainer, an auth bootstrap
//! script, a verify script, and a CI workflow stub via
//! `dex_core::agent_env::{plan_agent_env_init, apply_agent_env_plan}`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;

use dex_core::agent_env::{AgentEnvManifest, FileState, apply_agent_env_plan, plan_agent_env_init};
use dex_core::error::DexError;

#[derive(Args)]
pub struct AgentEnvArgs {
    #[command(subcommand)]
    pub cmd: AgentEnvCommand,
}

#[derive(Subcommand)]
pub enum AgentEnvCommand {
    /// Generate .devcontainer/, auth bootstrap, verify, and CI files from dex.agent-env.toml.
    Init(InitArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Project directory containing dex.agent-env.toml.
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Show what would be written without modifying any files.
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: AgentEnvArgs) -> Result<(), DexError> {
    match args.cmd {
        AgentEnvCommand::Init(a) => run_init(a),
    }
}

fn run_init(args: InitArgs) -> Result<(), DexError> {
    let project_dir = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    let manifest_path = project_dir.join("dex.agent-env.toml");
    let manifest = AgentEnvManifest::from_path(&manifest_path)?;
    manifest.validate()?;

    println!("\n{}\n", style("dex agent-env init").bold());

    let plan = plan_agent_env_init(&manifest, &project_dir)?;

    if args.dry_run {
        for file in &plan.files {
            let verb = match file.state {
                FileState::Created => "create",
                FileState::Updated => "update",
                FileState::Unchanged => "unchanged",
            };
            println!(
                "  {} would {} {}",
                style("•").cyan(),
                verb,
                style(file.path.display()).dim()
            );
        }
        println!(
            "\n{}",
            style("Dry run — no files were modified. Re-run without --dry-run to apply.").dim()
        );
    } else {
        apply_agent_env_plan(&plan)?;
        let mut created = 0usize;
        let mut updated = 0usize;
        for file in &plan.files {
            let verb = match file.state {
                FileState::Created => {
                    created += 1;
                    "created"
                }
                FileState::Updated => {
                    updated += 1;
                    "updated"
                }
                FileState::Unchanged => "unchanged",
            };
            println!(
                "  {} {} {}",
                style("✓").green(),
                verb,
                style(file.path.display()).dim()
            );
        }
        println!(
            "\n{} {} created, {} updated, {} unchanged.",
            style("Done!").green().bold(),
            created,
            updated,
            plan.files.len() - created - updated
        );
    }

    Ok(())
}

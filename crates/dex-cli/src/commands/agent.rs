//! `dex agent` — agent project scaffolding for Databricks.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Confirm, Input, Select};

use dex_core::DexError;
use dex_core::agent::{AgentAnswers, AgentDeployTarget, AgentTrigger, scaffold_agent};

use crate::output;

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Subcommand)]
enum AgentCommand {
    /// Scaffold a new agent project via interactive Q&A.
    New(AgentNewArgs),
}

#[derive(Args)]
struct AgentNewArgs {
    /// Agent name (skips first prompt).
    #[arg(short, long)]
    name: Option<String>,

    /// Parent directory.
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Skip the generative phase (Claude).
    #[arg(long)]
    no_generate: bool,
}

pub fn run(args: AgentArgs) -> Result<(), DexError> {
    match args.command {
        AgentCommand::New(new_args) => run_new(new_args),
    }
}

fn run_new(args: AgentNewArgs) -> Result<(), DexError> {
    println!("\n{}\n", style("━━━ dex agent new ━━━").cyan().bold());

    // --- Q&A Flow ---
    let description = match &args.name {
        Some(n) => n.clone(),
        None => Input::<String>::new()
            .with_prompt("What does this agent do in one sentence?")
            .interact_text()
            .map_err(io_error)?,
    };

    let suggested_name = suggest_name(&description);
    let agent_name: String = Input::new()
        .with_prompt("Agent name")
        .default(suggested_name)
        .interact_text()
        .map_err(io_error)?;

    let trigger_choices = ["user request", "schedule", "event", "upstream system"];
    let trigger_idx = Select::new()
        .with_prompt("What triggers it?")
        .items(&trigger_choices)
        .default(0)
        .interact()
        .map_err(io_error)?;
    let trigger = match trigger_idx {
        0 => AgentTrigger::UserRequest,
        1 => AgentTrigger::Schedule,
        2 => AgentTrigger::Event,
        3 => AgentTrigger::UpstreamSystem,
        _ => AgentTrigger::UserRequest,
    };

    let success: String = Input::new()
        .with_prompt("What does success look like?")
        .interact_text()
        .map_err(io_error)?;

    let reads: String = Input::new()
        .with_prompt("What does it need to read?")
        .interact_text()
        .map_err(io_error)?;

    let writes: String = Input::new()
        .with_prompt("What does it need to write or change?")
        .interact_text()
        .map_err(io_error)?;

    let handoff = Confirm::new()
        .with_prompt("Does it hand off to a human or another agent?")
        .default(false)
        .interact()
        .map_err(io_error)?;

    let autonomous = Confirm::new()
        .with_prompt("Should it act autonomously?")
        .default(true)
        .interact()
        .map_err(io_error)?;

    let example_input: String = Input::new()
        .with_prompt("Example input")
        .interact_text()
        .map_err(io_error)?;

    let example_output: String = Input::new()
        .with_prompt("What should the correct behavior/output be?")
        .interact_text()
        .map_err(io_error)?;

    let bad_output: String = Input::new()
        .with_prompt("What would a bad or dangerous output look like?")
        .interact_text()
        .map_err(io_error)?;

    let deploy_choices = ["job", "serving endpoint", "interactive"];
    let deploy_idx = Select::new()
        .with_prompt("How should it be deployed?")
        .items(&deploy_choices)
        .default(0)
        .interact()
        .map_err(io_error)?;
    let deploy_target = match deploy_idx {
        0 => AgentDeployTarget::Job,
        1 => AgentDeployTarget::ServingEndpoint,
        2 => AgentDeployTarget::Interactive,
        _ => AgentDeployTarget::Job,
    };

    // --- Scaffold (deterministic phase) ---
    let target = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    let answers = AgentAnswers {
        name: agent_name,
        description,
        trigger,
        success_criteria: success,
        reads,
        writes,
        handoff,
        autonomous,
        example_input,
        example_output,
        bad_output,
        deploy_target,
    };

    let result = scaffold_agent(&answers, &target)?;

    output::print_files_created(&result.files_created);

    // --- Generative phase (future) ---
    if !args.no_generate {
        output::print_dim(
            "Generative phase (Claude API) not yet implemented. \
             Use --no-generate or flesh out agent.py manually.",
        );
    }

    let project_name = result
        .project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    println!(
        "\n{} cd {project_name} && dex deploy\n",
        style("Done.").green().bold()
    );

    Ok(())
}

/// Suggest a project name from a description.
fn suggest_name(description: &str) -> String {
    let skip = [
        "a", "an", "the", "and", "or", "for", "to", "in", "on", "that", "which", "is",
    ];
    let lower = description.to_lowercase();
    let meaningful: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| !skip.contains(w))
        .take(4)
        .collect();
    let name = meaningful.join("-");
    name.trim_end_matches('.').to_string()
}

fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
}

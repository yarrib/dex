//! `dex agent` — scaffold and manage AI agent projects.
//!
//! Thin wrapper over `dex init` that restricts the template to the `agent-*`
//! family and prompts for the SDK choice up front.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use dialoguer::Select;

use dex_core::DexError;
use dex_core::error::ConfigError;

use crate::commands::init::{self, InitArgs};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Scaffold a new AI agent project.
    New(NewArgs),
}

#[derive(Args)]
pub struct NewArgs {
    /// SDK to use for the agent. If omitted, you'll be prompted.
    #[arg(long, value_enum)]
    sdk: Option<Sdk>,

    /// Target directory.
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Use defaults for all variables (non-interactive).
    #[arg(long)]
    no_prompt: bool,

    /// TOML file of pre-filled variable values.
    #[arg(long)]
    standards: Option<PathBuf>,

    /// Named preset profile to load.
    #[arg(long)]
    preset: Option<String>,

    /// TOML presets file to use instead of the default location.
    #[arg(long)]
    presets_file: Option<PathBuf>,

    /// Load pre-filled variable values from a saved answers file.
    #[arg(long)]
    answers: Option<PathBuf>,

    /// Save answered variable values to a TOML file after scaffold.
    #[arg(long, short = 's', num_args = 0..=1, default_missing_value = "")]
    save_answers: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Sdk {
    Anthropic,
    Openai,
    Baml,
}

impl Sdk {
    fn template_name(self) -> &'static str {
        match self {
            Sdk::Anthropic => "agent-anthropic",
            Sdk::Openai => "agent-openai",
            Sdk::Baml => "agent-baml",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Sdk::Anthropic => "anthropic — Anthropic SDK (claude-* models)",
            Sdk::Openai => "openai — OpenAI SDK (gpt-* models)",
            Sdk::Baml => "baml — BAML typed functions (any provider)",
        }
    }
}

pub fn run(args: AgentArgs) -> Result<(), DexError> {
    match args.command {
        AgentCommand::New(a) => run_new(a),
    }
}

fn run_new(args: NewArgs) -> Result<(), DexError> {
    let sdk = match args.sdk {
        Some(s) => s,
        None if args.no_prompt => Sdk::Anthropic,
        None => prompt_sdk()?,
    };

    let init_args = InitArgs {
        template: sdk.template_name().to_string(),
        dir: args.dir,
        no_prompt: args.no_prompt,
        standards: args.standards,
        preset: args.preset,
        presets_file: args.presets_file,
        answers: args.answers,
        save_answers: args.save_answers,
    };

    init::run(init_args)
}

fn prompt_sdk() -> Result<Sdk, DexError> {
    let options = [Sdk::Anthropic, Sdk::Openai, Sdk::Baml];
    let labels: Vec<&str> = options.iter().map(|s| s.label()).collect();

    let selection = Select::new()
        .with_prompt("Choose SDK")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|e| DexError::Config(ConfigError::Invalid(e.to_string())))?;

    Ok(options[selection])
}

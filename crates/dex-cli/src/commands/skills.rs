//! `dex skills` — manage and install AI agent skill packs.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Confirm, MultiSelect};

use dex_core::DexError;
use dex_core::config::{
    load_dex_config, load_project_config, resolve_skill_remote, user_config_path,
};
use dex_core::skills::{InstallTarget, install_skills, list_packs, load_pack_with_remote_fetch};

use crate::output;

// ---------------------------------------------------------------------------
// Arg types
// ---------------------------------------------------------------------------

#[derive(Args)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand)]
pub enum SkillsCommand {
    /// Interactively install skill packs into this project.
    Init(InitArgs),
    /// List available skill packs and their skills.
    List(ListArgs),
    /// Register a remote skill pack repository.
    Add(AddArgs),
    /// Sync installed skills based on dex.toml [skills] config.
    Sync(SyncArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Target directory (defaults to current directory).
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Install to specific targets (comma-separated: claude,cursor,copilot,generic).
    #[arg(long)]
    targets: Option<String>,

    /// Install specific packs (comma-separated). Skips interactive selection.
    #[arg(long)]
    packs: Option<String>,

    /// Auto-confirm all prompts (e.g. updating dex.toml). Requires --packs and --targets.
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show individual skills within each pack.
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
pub struct AddArgs {
    /// Git URL of the skill pack repository.
    url: String,
    /// Local name to refer to this pack source.
    #[arg(long)]
    name: Option<String>,
    /// Git ref (branch, tag, or commit) to pin.
    #[arg(long)]
    git_ref: Option<String>,
}

#[derive(Args)]
pub struct SyncArgs {
    /// Target directory containing dex.toml (defaults to current directory).
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Fetch latest from remote skill repositories before syncing.
    #[arg(long)]
    update: bool,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: SkillsArgs) -> Result<(), DexError> {
    match args.command {
        SkillsCommand::Init(a) => run_init(a),
        SkillsCommand::List(a) => run_list(a),
        SkillsCommand::Add(a) => run_add(a),
        SkillsCommand::Sync(a) => run_sync(a),
    }
}

// ---------------------------------------------------------------------------
// Template-driven install (library API path, no shell)
// ---------------------------------------------------------------------------

/// Install the given skill packs into `target_dir` for the given targets, and
/// record the choice in `dex.toml`. Invoked by the scaffold flow (`dex init` and
/// the MCP `scaffold_agent` tool) so the output is identical on both paths.
///
/// `quiet` suppresses per-file output; a one-line summary is still printed.
pub fn install_template_skills(
    target_dir: &std::path::Path,
    packs: &[String],
    targets: &[InstallTarget],
    quiet: bool,
) -> Result<usize, DexError> {
    let config = load_dex_config();
    let mut total = 0usize;

    for pack_name in packs {
        match load_pack_with_remote_fetch(
            pack_name,
            config.skills_dir.as_deref(),
            &config.skill_remotes,
            false,
        ) {
            Ok(pack) => {
                let result = install_skills(&pack, target_dir, targets)?;
                total += result.files_written.len();
                if !quiet {
                    println!(
                        "  {} {} ({} files)",
                        style("✓").green(),
                        style(pack_name).cyan(),
                        result.files_written.len()
                    );
                }
            }
            Err(e) => {
                output::print_warning(&format!("could not load pack '{pack_name}': {e}"));
            }
        }
    }

    let dex_toml = target_dir.join("dex.toml");
    if dex_toml.exists() {
        write_skills_to_dex_toml(&dex_toml, packs, targets)?;
    }

    Ok(total)
}

// ---------------------------------------------------------------------------
// dex skills init
// ---------------------------------------------------------------------------

fn run_init(args: InitArgs) -> Result<(), DexError> {
    if args.yes && (args.packs.is_none() || args.targets.is_none()) {
        return Err(DexError::Config(dex_core::error::ConfigError::Invalid(
            "--yes requires both --packs and --targets (non-interactive mode cannot prompt)".into(),
        )));
    }

    let target_dir = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    println!("\n{}\n", style("dex skills init").bold());

    let config = load_dex_config();

    // Fetch remotes to cache before listing.
    for remote in &config.skill_remotes {
        if let Err(e) = resolve_skill_remote(remote, false) {
            output::print_warning(&format!("could not fetch remote '{}': {e}", remote.name));
        }
    }

    let available = list_packs(config.skills_dir.as_deref(), &config.skill_remotes);

    if available.is_empty() {
        println!(
            "{}",
            style("No skill packs available. Add a remote with `dex skills add <url>`.").dim()
        );
        return Ok(());
    }

    // Select packs.
    let selected_packs: Vec<String> = if let Some(packs_arg) = &args.packs {
        packs_arg.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        let pack_labels: Vec<String> = available
            .iter()
            .map(|e| format!("{} — {}", e.name, e.description))
            .collect();

        let selections = MultiSelect::new()
            .with_prompt("Select skill packs to install (space to toggle, enter to confirm)")
            .items(&pack_labels)
            .interact()
            .map_err(io_error)?;

        if selections.is_empty() {
            println!("{}", style("No packs selected.").dim());
            return Ok(());
        }

        selections
            .into_iter()
            .map(|i| available[i].name.clone())
            .collect()
    };

    // Select install targets.
    let selected_targets: Vec<InstallTarget> = if let Some(targets_arg) = &args.targets {
        targets_arg
            .split(',')
            .map(|s| InstallTarget::parse(s.trim()))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let all_targets = [
            InstallTarget::Claude,
            InstallTarget::Cursor,
            InstallTarget::Copilot,
            InstallTarget::Generic,
        ];
        let labels: Vec<&str> = all_targets.iter().map(|t| t.display_name()).collect();

        let selections = MultiSelect::new()
            .with_prompt("Select install targets")
            .items(&labels)
            .defaults(&[true, false, false, false]) // Claude on by default
            .interact()
            .map_err(io_error)?;

        if selections.is_empty() {
            println!("{}", style("No targets selected.").dim());
            return Ok(());
        }

        selections.into_iter().map(|i| all_targets[i]).collect()
    };

    // Load and install each selected pack.
    let mut total_written = 0usize;

    for pack_name in &selected_packs {
        match load_pack_with_remote_fetch(
            pack_name,
            config.skills_dir.as_deref(),
            &config.skill_remotes,
            false,
        ) {
            Ok(pack) => {
                let result = install_skills(&pack, &target_dir, &selected_targets)?;
                total_written += result.files_written.len();
                println!(
                    "  {} {} ({} files)",
                    style("✓").green(),
                    style(pack_name).cyan(),
                    result.files_written.len()
                );
                for f in &result.files_written {
                    println!("    {}", style(f.display()).dim());
                }
            }
            Err(e) => {
                output::print_warning(&format!("could not load pack '{pack_name}': {e}"));
            }
        }
    }

    println!(
        "\n{} {} skill file{} installed.\n",
        style("Done!").green().bold(),
        total_written,
        if total_written == 1 { "" } else { "s" }
    );

    // Offer to update dex.toml.
    let dex_toml = target_dir.join("dex.toml");
    if dex_toml.exists() {
        let update = if args.yes {
            true
        } else {
            Confirm::new()
                .with_prompt("Update dex.toml [skills] with selected packs and targets?")
                .default(true)
                .interact()
                .map_err(io_error)?
        };

        if update {
            write_skills_to_dex_toml(&dex_toml, &selected_packs, &selected_targets)?;
            println!("  {} dex.toml updated.", style("✓").green());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// dex skills list
// ---------------------------------------------------------------------------

fn run_list(args: ListArgs) -> Result<(), DexError> {
    let config = load_dex_config();
    let packs = list_packs(config.skills_dir.as_deref(), &config.skill_remotes);

    if packs.is_empty() {
        println!(
            "{}",
            style("No skill packs available. Add a remote with `dex skills add <url>`.").dim()
        );
        return Ok(());
    }

    println!("\n{}\n", style("Available skill packs:").bold());

    for entry in &packs {
        let source_label = match &entry.source {
            dex_core::SkillSource::Embedded => style("built-in").dim().to_string(),
            dex_core::SkillSource::Directory(p) => style(p.display().to_string()).dim().to_string(),
        };

        println!(
            "  {} {} {}",
            style(&entry.name).cyan().bold(),
            style(format!("v{}", entry.version)).dim(),
            style(format!("({})", source_label)).dim()
        );
        println!("  {}", entry.description);

        if args.verbose {
            // Load the pack to show individual skills.
            if let Ok(pack) = load_pack_with_remote_fetch(
                &entry.name,
                config.skills_dir.as_deref(),
                &config.skill_remotes,
                false,
            ) {
                for skill in &pack.manifest.skills {
                    println!(
                        "    {} [{}] {}",
                        style(&skill.name).white(),
                        style(skill.skill_type.to_string()).dim(),
                        skill.description
                    );
                }
            }
        }

        println!();
    }

    println!(
        "{}",
        style(
            "Install packs with `dex skills init`. Add remote packs with `dex skills add <url>`."
        )
        .dim()
    );
    println!();

    Ok(())
}

// ---------------------------------------------------------------------------
// dex skills add
// ---------------------------------------------------------------------------

fn run_add(args: AddArgs) -> Result<(), DexError> {
    // Derive name from URL if not specified.
    let name = args.name.unwrap_or_else(|| {
        args.url
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("remote")
            .trim_end_matches(".git")
            .to_string()
    });

    println!(
        "\n{} Adding skill pack remote '{}'\n",
        style("dex skills add").bold(),
        style(&name).cyan()
    );

    // Append to ~/.config/dex/config.toml.
    let config_path = user_config_path();

    // Read existing config.
    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path).map_err(|source| DexError::Io {
            path: config_path.clone(),
            source,
        })?
    } else {
        String::new()
    };

    // Build the new remote entry in TOML.
    let ref_line = args
        .git_ref
        .as_deref()
        .map(|r| format!("\nref = \"{}\"", r))
        .unwrap_or_default();

    let new_entry = format!(
        "\n[[skills.remotes]]\nname = \"{}\"\nurl  = \"{}\"{}",
        name, args.url, ref_line
    );

    let updated = existing + &new_entry + "\n";

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    std::fs::write(&config_path, updated).map_err(|source| DexError::Io {
        path: config_path.clone(),
        source,
    })?;

    println!(
        "  {} Registered '{}' in {}",
        style("✓").green(),
        name,
        config_path.display()
    );
    println!("  {} Fetching pack...", style("→").dim());

    // Immediately fetch to cache.
    let remote = dex_core::config::RemoteSource {
        name: name.clone(),
        url: args.url.clone(),
        git_ref: args.git_ref,
    };

    match resolve_skill_remote(&remote, false) {
        Ok(path) => {
            println!("  {} Cached at {}", style("✓").green(), path.display());
        }
        Err(e) => {
            output::print_warning(&format!("could not fetch remote: {e}"));
            println!("  Run `dex skills init` after network access is restored.");
        }
    }

    println!(
        "\n  Run {} to install skills from this pack.\n",
        style("dex skills init").cyan()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// dex skills sync
// ---------------------------------------------------------------------------

fn run_sync(args: SyncArgs) -> Result<(), DexError> {
    let target_dir = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    println!("\n{}\n", style("dex skills sync").bold());

    let dex_toml = target_dir.join("dex.toml");
    let project_config = load_project_config(&dex_toml)?;

    let skills_config = match project_config.skills {
        Some(s) if !s.packs.is_empty() => s,
        _ => {
            println!(
                "{}",
                style("No [skills] section in dex.toml. Run `dex skills init` to configure.").dim()
            );
            return Ok(());
        }
    };

    let config = load_dex_config();

    // Fetch remotes if requested.
    if args.update {
        for remote in &config.skill_remotes {
            if let Err(e) = resolve_skill_remote(remote, true) {
                output::print_warning(&format!("could not update remote '{}': {e}", remote.name));
            }
        }
    }

    let targets: Vec<InstallTarget> = skills_config
        .targets
        .iter()
        .map(|s| InstallTarget::parse(s))
        .collect::<Result<Vec<_>, _>>()?;

    if targets.is_empty() {
        output::print_warning("dex.toml [skills].targets is empty — nothing to install into.");
        return Ok(());
    }

    let mut total_written = 0usize;

    for pack_name in &skills_config.packs {
        match load_pack_with_remote_fetch(
            pack_name,
            config.skills_dir.as_deref(),
            &config.skill_remotes,
            false,
        ) {
            Ok(pack) => {
                let result = install_skills(&pack, &target_dir, &targets)?;
                total_written += result.files_written.len();
                println!(
                    "  {} {} ({} files)",
                    style("✓").green(),
                    style(pack_name).cyan(),
                    result.files_written.len()
                );
                for f in &result.files_written {
                    println!("    {}", style(f.display()).dim());
                }
            }
            Err(e) => {
                output::print_warning(&format!("could not load pack '{pack_name}': {e}"));
            }
        }
    }

    println!(
        "\n{} {} skill file{} synced.\n",
        style("Done!").green().bold(),
        total_written,
        if total_written == 1 { "" } else { "s" }
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Append or update the `[skills]` section in `dex.toml`.
fn write_skills_to_dex_toml(
    path: &std::path::Path,
    packs: &[String],
    targets: &[InstallTarget],
) -> Result<(), DexError> {
    let existing = std::fs::read_to_string(path).map_err(|source| DexError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    // Remove existing [skills] section if present (naive but safe for TOML).
    let without_skills = remove_toml_section(&existing, "skills");

    let target_strs: Vec<String> = targets
        .iter()
        .map(|t| format!("\"{}\"", t.as_str()))
        .collect();
    let pack_strs: Vec<String> = packs.iter().map(|p| format!("\"{}\"", p)).collect();

    let skills_section = format!(
        "\n[skills]\npacks   = [{}]\ntargets = [{}]\n",
        pack_strs.join(", "),
        target_strs.join(", ")
    );

    let updated = without_skills + &skills_section;
    std::fs::write(path, updated).map_err(|source| DexError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Remove a top-level `[section]` block from TOML text (removes until next `[` or EOF).
fn remove_toml_section(content: &str, section: &str) -> String {
    let header = format!("[{}]", section);
    let mut result = String::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            in_section = false;
        }
        if !in_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
}

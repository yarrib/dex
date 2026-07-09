//! `dex update` — re-apply template changes to an already-generated project.
//!
//! Renders the template at the recorded ref (from `.dex/cache/baseline/`) and
//! at the target ref, then 3-way merges the delta into the working tree. Clean
//! hunks apply silently; conflicts are written with standard git markers for
//! the user to resolve. Local edits are never silently overwritten.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use clap::Args;
use console::style;
use dialoguer::Confirm;

use dex_core::update::{apply_update, load_state_manifest, plan_update, resolve_new_template};
use dex_core::{DexError, Template, UpdateReport};

use crate::commands::prompting::{
    evaluate_when, io_error, prompt_variable, skipped_default, toml_val_to_minijinja,
    toml_value_to_string,
};
use crate::output;

#[derive(Args)]
pub struct UpdateArgs {
    /// Target template ref (tag, commit, or version). Defaults to the latest
    /// available for the recorded source.
    #[arg(long = "ref")]
    pub git_ref: Option<String>,

    /// Preview the changes without writing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Use defaults for any newly-introduced variables (non-interactive).
    #[arg(long)]
    pub no_prompt: bool,

    /// Project directory to update.
    #[arg(short, long, default_value = ".")]
    pub dir: String,
}

pub fn run(args: UpdateArgs) -> Result<(), DexError> {
    let project_dir = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    // Reading state fails with a helpful `NoManifest` error for pre-feature
    // projects.
    let manifest = load_state_manifest(&project_dir)?;

    println!(
        "\n{} — {}\n",
        style("dex update").bold(),
        style(&manifest.template.name).cyan()
    );

    // The context hooks and prompts render against: the answers on record.
    let hook_ctx: HashMap<String, minijinja::Value> = manifest
        .answers
        .iter()
        .map(|(k, v)| (k.clone(), toml_val_to_minijinja(v)))
        .collect();

    // pre_update runs before we touch anything (skipped on --dry-run).
    if !args.dry_run {
        run_hook(
            "pre_update",
            manifest.hooks.pre_update.as_deref(),
            &project_dir,
            &hook_ctx,
            args.no_prompt,
        )?;
    }

    let resolved = resolve_new_template(&manifest, args.git_ref.as_deref())?;

    let variables =
        resolve_update_variables(&resolved.template, &manifest.answers, args.no_prompt)?;

    let plan = plan_update(&project_dir, &manifest, &resolved, &variables)?;

    if plan.is_noop() {
        println!(
            "  {} Already up to date (ref {}).\n",
            style("✓").green().bold(),
            style(short(&manifest.template.git_ref)).cyan()
        );
        return Ok(());
    }

    if args.dry_run {
        print_report(&plan.report, true);
        println!(
            "  {} dry run — no files were changed.\n",
            style("note:").yellow().bold()
        );
        return Ok(());
    }

    apply_update(&project_dir, &manifest, &plan, env!("CARGO_PKG_VERSION"))?;
    print_report(&plan.report, false);

    // post_update runs the hooks carried forward from the new template.
    run_hook(
        "post_update",
        plan.new_hooks.post_update.as_deref(),
        &project_dir,
        &variables,
        args.no_prompt,
    )?;

    Ok(())
}

/// Execute an update hook command in the project directory. Rendered through
/// Jinja against the answer context, confirmed unless `--no-prompt`, and
/// non-fatal on failure — same model as `dex init`'s `[on_success]` hook.
fn run_hook(
    label: &str,
    command: Option<&str>,
    project_dir: &Path,
    ctx: &HashMap<String, minijinja::Value>,
    no_prompt: bool,
) -> Result<(), DexError> {
    let Some(raw) = command else {
        return Ok(());
    };
    if raw.trim().is_empty() {
        return Ok(());
    }

    let env = minijinja::Environment::new();
    let cmd = env.render_str(raw, ctx).unwrap_or_else(|_| raw.to_string());

    let should_run = if no_prompt {
        true
    } else {
        Confirm::new()
            .with_prompt(format!("Run {label} hook `{cmd}` now?"))
            .default(true)
            .interact()
            .map_err(io_error)?
    };
    if !should_run {
        return Ok(());
    }

    println!(
        "  {} {}",
        style(format!("{label}:")).cyan().bold(),
        style(&cmd).dim()
    );

    let mut parts = cmd.split_whitespace();
    let Some(program) = parts.next() else {
        return Ok(());
    };
    let args: Vec<&str> = parts.collect();
    match std::process::Command::new(program)
        .args(&args)
        .current_dir(project_dir)
        .status()
    {
        Ok(s) if s.success() => {}
        Ok(s) => output::print_warning(&format!("{label} hook `{cmd}` exited with status {s}")),
        Err(e) => output::print_warning(&format!("could not run {label} hook `{cmd}`: {e}")),
    }
    Ok(())
}

/// Resolve the variable context for the update: known answers are taken
/// verbatim; variables new to the target template are prompted for (or take
/// their defaults under `--no-prompt`).
fn resolve_update_variables(
    template: &Template,
    known: &BTreeMap<String, toml::Value>,
    no_prompt: bool,
) -> Result<HashMap<String, minijinja::Value>, DexError> {
    let mut variables: HashMap<String, minijinja::Value> = HashMap::new();
    let mut announced_new = false;

    for spec in &template.variables {
        let effective_default = spec
            .default
            .as_ref()
            .map(toml_value_to_string)
            .unwrap_or_default();

        // Honor `when` against already-resolved variables.
        if let Some(when_expr) = &spec.when
            && !evaluate_when(when_expr, &variables)
        {
            variables.insert(spec.name.clone(), skipped_default(spec, &effective_default));
            continue;
        }

        // Known answer from the manifest — reuse without prompting.
        if let Some(val) = known.get(&spec.name) {
            variables.insert(spec.name.clone(), toml_val_to_minijinja(val));
            continue;
        }

        // A variable new since the project was generated.
        if !no_prompt && !announced_new {
            println!(
                "  {} answering new template variables:\n",
                style("new:").cyan().bold()
            );
            announced_new = true;
        }
        let val = prompt_variable(spec, &effective_default, no_prompt)?;
        variables.insert(spec.name.clone(), val);
    }

    Ok(variables)
}

fn print_report(report: &UpdateReport, dry_run: bool) {
    let verb = if dry_run { "would change" } else { "changed" };
    println!(
        "  {} {} → {}",
        style("update:").cyan().bold(),
        style(short(&report.old_ref)).dim(),
        style(short(&report.new_ref)).cyan()
    );

    print_group("added", &report.added, "green");
    print_group("updated", &report.updated, "green");
    print_group("merged", &report.merged, "cyan");
    print_group("deleted", &report.deleted, "yellow");

    if report.has_conflicts() {
        println!(
            "\n  {} {} file(s) with conflicts — resolve the markers, then commit:",
            style("conflicts:").red().bold(),
            report.conflicts.len()
        );
        for path in &report.conflicts {
            println!("    {} {}", style("!").red().bold(), path.display());
        }
    }

    for notice in &report.notices {
        output::print_warning(notice);
    }

    let n = report.files_changed();
    println!(
        "\n  {} {n} file(s) {verb}{}.\n",
        style("✓").green().bold(),
        if report.has_conflicts() {
            format!(", {} with conflicts", report.conflicts.len())
        } else {
            String::new()
        }
    );
}

fn print_group(label: &str, paths: &[PathBuf], color: &str) {
    if paths.is_empty() {
        return;
    }
    let styled = match color {
        "green" => style(label).green().bold(),
        "yellow" => style(label).yellow().bold(),
        _ => style(label).cyan().bold(),
    };
    println!("\n  {styled}:");
    for path in paths {
        println!("    {} {}", style("•").dim(), path.display());
    }
}

/// Shorten a long ref (commit SHA) for display; leaves versions/short refs be.
fn short(git_ref: &str) -> String {
    if git_ref.len() > 12 && git_ref.chars().all(|c| c.is_ascii_hexdigit()) {
        git_ref[..12].to_string()
    } else {
        git_ref.to_string()
    }
}

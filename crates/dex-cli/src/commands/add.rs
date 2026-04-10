//! `dex add <trait>` — bolt a composable trait onto an existing project.

use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;
use console::style;
use dialoguer::{Confirm, Input, Select};

use dex_core::DexError;
use dex_core::apply_trait;
use dex_core::config::{load_project_config, record_trait};
use dex_core::error::ConfigError;
use dex_core::template::variables::VariableType;
use dex_core::traits::{list_traits, load_trait};

use crate::output;

#[derive(Args)]
pub struct AddArgs {
    /// Trait to add (e.g. `docker`, `ci-github`). Omit with --list to see all.
    trait_name: Option<String>,

    /// Project directory (must contain a dex.toml).
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Use defaults for all variables (non-interactive).
    #[arg(long)]
    no_prompt: bool,

    /// Preview what would be created/patched without writing anything.
    #[arg(long)]
    dry_run: bool,

    /// List all available traits and exit.
    #[arg(long)]
    list: bool,
}

pub fn run(args: AddArgs) -> Result<(), DexError> {
    if args.list {
        return list();
    }

    let trait_name = args.trait_name.as_deref().ok_or_else(|| {
        DexError::Config(ConfigError::Invalid(
            "trait name required. Run `dex add --list` to see available traits.".to_string(),
        ))
    })?;

    let target = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    // Require an existing dex.toml — `dex add` only works inside a dex project.
    let dex_toml_path = target.join("dex.toml");
    let project_config = load_project_config(&dex_toml_path)
        .map_err(|_| DexError::Config(ConfigError::NotFound(dex_toml_path.clone())))?;

    // Check if the trait has already been applied.
    if project_config
        .project
        .traits
        .contains(&trait_name.to_string())
    {
        return Err(DexError::Config(ConfigError::Invalid(format!(
            "trait '{}' is already applied to this project. \
             Remove it from [project].traits in dex.toml if you want to re-apply it.",
            trait_name
        ))));
    }

    // Load the trait (embedded first, no custom dir in v1).
    let t = load_trait(trait_name, None)?;

    println!(
        "\n{} — adding trait {}\n",
        style("dex add").bold(),
        style(trait_name).cyan()
    );

    // Collect variable values, inheriting project_name / python_version from
    // the existing project config where possible.
    let project_name = project_config.project.name.clone();
    let mut variables: HashMap<String, minijinja::Value> = HashMap::new();

    // Pre-seed project_name from dex.toml so traits can reference it in patches.
    variables.insert(
        "project_name".to_string(),
        minijinja::Value::from(project_name.clone()),
    );

    for spec in &t.variables {
        let effective_default = spec
            .default
            .as_ref()
            .map(toml_value_to_string)
            .unwrap_or_default();

        // Skip this variable if its `when` condition evaluates to false.
        if let Some(when_expr) = &spec.when
            && !evaluate_when(when_expr, &variables)
        {
            let default_val = match spec.var_type {
                VariableType::Bool => {
                    let b = effective_default.is_empty() || effective_default == "true";
                    minijinja::Value::from(b)
                }
                _ => minijinja::Value::from(effective_default),
            };
            variables.insert(spec.name.clone(), default_val);
            continue;
        }

        if args.no_prompt || args.dry_run {
            let val = match spec.var_type {
                VariableType::Bool => {
                    let b = effective_default.is_empty() || effective_default == "true";
                    minijinja::Value::from(b)
                }
                VariableType::Choice => {
                    let v = if effective_default.is_empty() {
                        spec.choices
                            .as_ref()
                            .and_then(|c| c.first().cloned())
                            .unwrap_or_default()
                    } else {
                        effective_default
                    };
                    minijinja::Value::from(v)
                }
                _ => minijinja::Value::from(effective_default),
            };
            variables.insert(spec.name.clone(), val);
        } else {
            let val = match spec.var_type {
                VariableType::Choice => {
                    let choices = spec.choices.as_deref().unwrap_or(&[]);
                    let default_idx = choices
                        .iter()
                        .position(|c| c == &effective_default)
                        .unwrap_or(0);
                    let selection = Select::new()
                        .with_prompt(&spec.prompt)
                        .items(choices)
                        .default(default_idx)
                        .interact()
                        .map_err(io_error)?;
                    minijinja::Value::from(choices[selection].clone())
                }
                VariableType::Bool => {
                    let default = effective_default.is_empty() || effective_default == "true";
                    let answer = Confirm::new()
                        .with_prompt(&spec.prompt)
                        .default(default)
                        .interact()
                        .map_err(io_error)?;
                    minijinja::Value::from(answer)
                }
                _ => {
                    let mut input = Input::<String>::new().with_prompt(&spec.prompt);
                    if !effective_default.is_empty() {
                        input = input.default(effective_default.clone());
                    }
                    let answer = input.interact_text().map_err(io_error)?;
                    minijinja::Value::from(answer)
                }
            };
            variables.insert(spec.name.clone(), val);
        }
    }

    if args.dry_run {
        println!(
            "{}",
            style("Dry run — no files will be written.\n").yellow()
        );
        println!("Files that would be created:");
        for rel_path in t.files.keys() {
            println!("  {} {}", style("+").green(), rel_path.display());
        }
        if !t.patches.is_empty() {
            println!("\nFiles that would be patched:");
            for patch in &t.patches {
                println!("  {} {}", style("~").cyan(), patch.target);
            }
        }
        return Ok(());
    }

    // Apply the trait.
    let result = apply_trait(&t, &target, &variables)?;

    // Record in dex.toml.
    record_trait(&target, trait_name)?;

    // Print summary.
    output::print_files_created(&result.files_created);

    if !result.files_patched.is_empty() {
        for path in &result.files_patched {
            println!("  {} {}", style("patched").cyan(), path.display());
        }
    }

    if !result.files_skipped.is_empty() {
        for path in &result.files_skipped {
            println!(
                "  {} {} (already exists, skipped)",
                style("skipped").yellow(),
                path.display()
            );
        }
    }

    println!(
        "\n{} trait '{}' applied.",
        style("✓").green().bold(),
        style(trait_name).cyan()
    );

    Ok(())
}

/// List available traits (used by `dex add --list`).
pub fn list() -> Result<(), DexError> {
    let traits = list_traits(None)?;
    if traits.is_empty() {
        println!("No traits available.");
        return Ok(());
    }
    println!("{}", style("Available traits:").bold());
    for t in &traits {
        println!("  {:20} {}", style(&t.name).cyan(), t.description);
    }
    Ok(())
}

fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        other => other.to_string(),
    }
}

fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
}

/// Evaluate a Jinja2 boolean expression against already-resolved variables.
///
/// Returns `true` if the expression evaluates to a truthy value, `false` otherwise
/// (including on evaluation errors, so a bad `when` expression silently skips the
/// variable rather than crashing the prompt loop).
fn evaluate_when(expr: &str, vars: &HashMap<String, minijinja::Value>) -> bool {
    let env = minijinja::Environment::new();
    let source = format!("{{% if {expr} %}}true{{% else %}}false{{% endif %}}");
    env.render_str(&source, vars)
        .is_ok_and(|r| r.trim() == "true")
}

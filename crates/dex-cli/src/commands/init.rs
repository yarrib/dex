//! `dex init` — scaffold a new project from a template.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::Args;
use console::style;
use dialoguer::{Confirm, Input, Select};

use dex_core::config::{
    load_answers, load_dex_config, load_preset, load_standards, resolve_remote, save_answers,
};
use dex_core::context_map::write_context_map;
use dex_core::template::TemplateSource;
use dex_core::template::registry::{list_templates, load_template};
use dex_core::template::variables::VariableType;
use dex_core::{DexError, scaffold};

use crate::output;

#[derive(Args)]
pub struct InitArgs {
    /// Template to scaffold from.
    #[arg(short, long, default_value = "default")]
    template: String,

    /// Target directory.
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Use defaults for all variables (non-interactive).
    #[arg(long)]
    no_prompt: bool,

    /// TOML file of pre-filled variable values (skips prompts for matched vars).
    #[arg(long)]
    standards: Option<PathBuf>,

    /// Named preset profile to load (from ~/.config/dex/presets.toml).
    #[arg(long)]
    preset: Option<String>,

    /// TOML presets file to use instead of the default location.
    #[arg(long)]
    presets_file: Option<PathBuf>,

    /// Load pre-filled variable values from a saved answers file (skips prompts for matched vars).
    #[arg(long)]
    answers: Option<PathBuf>,

    /// Save answered variable values to a TOML file after scaffold.
    /// Omit the path to use the default: ~/.config/dex/answers/<template>.toml
    #[arg(long, short = 's', num_args = 0..=1, default_missing_value = "")]
    save_answers: Option<String>,
}

pub fn run(args: InitArgs) -> Result<(), DexError> {
    let target = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir).to_path_buf());

    // Collect templates from all sources.
    let registry = collect_templates(None)?;

    println!(
        "\n{} — scaffolding with template {}\n",
        style("dex init").bold(),
        style(&args.template).cyan()
    );

    let entry = registry.get(&args.template).ok_or_else(|| {
        let available: Vec<_> = registry.keys().map(|s| s.as_str()).collect();
        DexError::Config(dex_core::error::ConfigError::Invalid(format!(
            "template '{}' not found. Available: {}",
            args.template,
            available.join(", ")
        )))
    })?;

    // Load preset profile (lower-priority pre-fills).
    let preset = if let Some(ref profile) = args.preset {
        load_preset(args.presets_file.as_deref(), profile)?
    } else {
        HashMap::new()
    };

    // Load standards (higher-priority pre-fills; override preset for the same key).
    let standards = load_standards(args.standards.as_deref())?;

    // Merge: preset as baseline, standards take precedence.
    let mut prefills = preset;
    for (k, v) in &standards {
        prefills.insert(k.clone(), v.clone());
    }

    // Load answers file (highest-priority pre-fills; overrides standards and preset).
    let answers_prefills = if let Some(ref path) = args.answers {
        load_answers(path)?
    } else {
        HashMap::new()
    };

    // Load the template.
    let template = load_template(&entry.source, &args.template)?;

    // Determine default project name from target directory.
    let default_project_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_project")
        .to_string();

    // Collect variables via interactive prompts.
    let mut variables: HashMap<String, minijinja::Value> = HashMap::new();

    for spec in &template.variables {
        let effective_default = if spec.name == "project_name" {
            default_project_name.clone()
        } else {
            spec.default
                .as_ref()
                .map(toml_value_to_string)
                .unwrap_or_default()
        };

        // Answers file (highest priority): overrides standards, preset, and defaults.
        if let Some(toml_val) = answers_prefills.get(&spec.name) {
            let mj_val = toml_val_to_minijinja(toml_val);
            variables.insert(spec.name.clone(), mj_val);
            continue;
        }

        // Pre-fill: preset or standards value skips the prompt entirely.
        if let Some(val) = prefills.get(&spec.name) {
            variables.insert(spec.name.clone(), minijinja::Value::from(val.clone()));
            continue;
        }

        if args.no_prompt {
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

                    // Add validation if a pattern is defined.
                    if let Some(pattern) = &spec.validate {
                        let re = regex::Regex::new(pattern).ok();
                        let pattern_str = pattern.clone();
                        input = input.validate_with(move |val: &String| -> Result<(), String> {
                            if let Some(ref re) = re {
                                if re.is_match(val) {
                                    Ok(())
                                } else {
                                    Err(format!(
                                        "value '{val}' does not match pattern '{pattern_str}'"
                                    ))
                                }
                            } else {
                                Ok(())
                            }
                        });
                    }

                    let answer = input.interact_text().map_err(io_error)?;
                    minijinja::Value::from(answer)
                }
            };
            variables.insert(spec.name.clone(), val);
        }
    }

    let result = scaffold(&template, &target, &variables)?;

    output::print_files_created(&result.files_created);

    // Write .context-map.json for AI agent consumption (best-effort; non-fatal).
    if let Err(e) = write_context_map(&result, &template, &target, &variables) {
        output::print_warning(&format!("could not write .context-map.json: {e}"));
    }

    // Write dex.toml so subsequent commands (dex add, dex skills sync, etc.) work.
    let dex_toml_path = target.join("dex.toml");
    if !dex_toml_path.exists() {
        let project_name = variables
            .get("project_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&args.template)
            .to_string();

        let mut dex_toml = format!(
            "[project]\nname = \"{project_name}\"\ntemplate = \"{}\"\n",
            args.template
        );

        if !template.suggested_skills.is_empty() {
            let packs = template
                .suggested_skills
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ");
            dex_toml.push_str(&format!("\n[skills]\npacks = [{packs}]\n"));
        }

        std::fs::write(&dex_toml_path, dex_toml).map_err(|source| DexError::Io {
            path: dex_toml_path.clone(),
            source,
        })?;

        println!(
            "  {} Wrote {}\n",
            console::style("created:").green(),
            console::style("dex.toml").cyan()
        );
    }

    // If the template suggests skill packs, print a hint.
    if !template.suggested_skills.is_empty() {
        println!(
            "  {} Suggested skill packs: {}\n  Run {} to install them.\n",
            console::style("tip:").yellow().bold(),
            console::style(template.suggested_skills.join(", ")).cyan(),
            console::style("dex skills init").cyan()
        );
    }

    // Save answers for future replay if --save-answers / -s was passed.
    // An empty string means "auto-name from template" (flag present, no path given).
    // Auto-named path: .dex/<template>.toml inside the scaffolded project directory.
    let resolved_save_path: Option<PathBuf> = args.save_answers.as_deref().map(|s| {
        if s.is_empty() {
            target.join(".dex").join(format!("{}.toml", args.template))
        } else {
            PathBuf::from(s)
        }
    });

    if let Some(ref save_path) = resolved_save_path {
        let typed_values: std::collections::HashMap<String, toml::Value> = template
            .variables
            .iter()
            .filter_map(|spec| {
                variables.get(&spec.name).map(|v| {
                    let toml_val = match spec.var_type {
                        VariableType::Bool => toml::Value::Boolean(v.is_true()),
                        _ => toml::Value::String(
                            v.as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| v.to_string()),
                        ),
                    };
                    (spec.name.clone(), toml_val)
                })
            })
            .collect();

        match save_answers(save_path, &args.template, &typed_values) {
            Ok(()) => {
                println!(
                    "  {} Answers saved to {}\n",
                    console::style("saved:").green(),
                    console::style(save_path.display()).cyan()
                );
            }
            Err(e) => {
                output::print_warning(&format!("could not save answers: {e}"));
            }
        }
    }

    // Post-scaffold activation hook.
    if let Some(on_success) = &result.on_success {
        run_on_success(on_success, &target, args.no_prompt)?;
    }

    Ok(())
}

/// Execute the `[on_success]` activation hook from the template.
///
/// Prints any `message`, then either auto-runs the command (`--no-prompt`) or
/// asks the user first. Failure is non-fatal: an error is printed and dex exits
/// successfully so the scaffold output isn't lost.
fn run_on_success(
    on_success: &dex_core::OnSuccessSpec,
    target: &Path,
    no_prompt: bool,
) -> Result<(), DexError> {
    if let Some(msg) = &on_success.message {
        println!(
            "  {} {}\n",
            console::style("next:").cyan().bold(),
            msg
        );
    }

    let Some(cmd) = &on_success.run else {
        return Ok(());
    };

    let should_run = if no_prompt {
        true
    } else {
        Confirm::new()
            .with_prompt(format!("Run `{cmd}` now?"))
            .default(true)
            .interact()
            .map_err(io_error)?
    };

    if !should_run {
        println!(
            "  {} Run {} manually when ready.\n",
            console::style("tip:").yellow().bold(),
            console::style(cmd).cyan()
        );
        return Ok(());
    }

    println!(
        "\n  {} {}\n",
        console::style("running:").green().bold(),
        console::style(cmd).cyan()
    );

    // Split the command into program + args for cross-platform compatibility.
    let mut parts = cmd.split_whitespace();
    let program = match parts.next() {
        Some(p) => p,
        None => return Ok(()),
    };
    let cmd_args: Vec<&str> = parts.collect();

    let status = std::process::Command::new(program)
        .args(&cmd_args)
        .current_dir(target)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(
                "\n  {} Setup complete.\n",
                console::style("done:").green().bold()
            );
        }
        Ok(s) => {
            output::print_warning(&format!(
                "`{cmd}` exited with status {s}. Run it manually if needed."
            ));
        }
        Err(e) => {
            output::print_warning(&format!(
                "could not run `{cmd}`: {e}. Run it manually if needed."
            ));
        }
    }

    Ok(())
}

/// Registry entry for a discovered template.
struct TemplateEntry {
    source: TemplateSource,
    #[allow(dead_code)]
    description: String,
}

/// Collect all available templates from embedded + config + extra_dir.
fn collect_templates(extra_dir: Option<&Path>) -> Result<HashMap<String, TemplateEntry>, DexError> {
    let mut registry = HashMap::new();

    // 1. Embedded templates (lowest priority).
    for meta in list_templates(&TemplateSource::Embedded)? {
        registry.insert(
            meta.name.clone(),
            TemplateEntry {
                source: TemplateSource::Embedded,
                description: meta.description,
            },
        );
    }

    // 2. Config-based sources.
    let config = load_dex_config();
    let mut dirs_to_scan: Vec<(PathBuf, TemplateSource)> = Vec::new();

    if let Some(dir) = &config.templates_dir {
        dirs_to_scan.push((dir.clone(), TemplateSource::Directory(dir.clone())));
    }

    for remote in &config.remotes {
        match resolve_remote(remote, true) {
            Ok(local) => {
                dirs_to_scan.push((local.clone(), TemplateSource::Directory(local)));
            }
            Err(e) => {
                output::print_warning(&format!("could not resolve remote '{}': {e}", remote.name));
            }
        }
    }

    // 3. Extra dir (from extension).
    if let Some(dir) = extra_dir {
        dirs_to_scan.push((
            dir.to_path_buf(),
            TemplateSource::Directory(dir.to_path_buf()),
        ));
    }

    for (_dir, source) in &dirs_to_scan {
        if let Ok(metas) = list_templates(source) {
            for meta in metas {
                registry.insert(
                    meta.name.clone(),
                    TemplateEntry {
                        source: source.clone(),
                        description: meta.description,
                    },
                );
            }
        }
    }

    Ok(registry)
}

/// Convert a typed TOML value from an answers file into a minijinja Value.
///
/// Preserves bool → bool and string → string so that template conditionals
/// behave correctly on replay.
fn toml_val_to_minijinja(v: &toml::Value) -> minijinja::Value {
    match v {
        toml::Value::Boolean(b) => minijinja::Value::from(*b),
        toml::Value::String(s) => minijinja::Value::from(s.clone()),
        toml::Value::Integer(i) => minijinja::Value::from(*i),
        toml::Value::Float(f) => minijinja::Value::from(*f),
        _ => minijinja::Value::from(v.to_string()),
    }
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

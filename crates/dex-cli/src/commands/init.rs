//! `dex init` — scaffold a new project from a template.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::Args;
use console::style;
use dialoguer::Confirm;

use dex_core::config::{
    RemoteSource, git_head_sha, load_answers, load_dex_config, load_preset, load_standards,
    remote_cache_dir, resolve_remote, save_answers,
};
use dex_core::context_map::write_context_map;
use dex_core::skills::InstallTarget;
use dex_core::template::TemplateSource;
use dex_core::template::registry::{list_templates, load_template};
use dex_core::template::variables::VariableType;
use dex_core::update::{SourceKind, record_project_state};
use dex_core::{DexError, Template, scaffold};

use crate::commands::prompting::{
    evaluate_when, prompt_variable, skipped_default, toml_val_to_minijinja, toml_value_to_string,
};
use crate::commands::skills::install_template_skills;
use crate::output;

#[derive(Args)]
pub struct InitArgs {
    /// Template to scaffold from.
    #[arg(short, long, default_value = "default")]
    pub template: String,

    /// Target directory.
    #[arg(short, long, default_value = ".")]
    pub dir: String,

    /// Use defaults for all variables (non-interactive).
    #[arg(long)]
    pub no_prompt: bool,

    /// TOML file of pre-filled variable values (skips prompts for matched vars).
    #[arg(long)]
    pub standards: Option<PathBuf>,

    /// Named preset profile to load (from ~/.config/dex/presets.toml).
    #[arg(long)]
    pub preset: Option<String>,

    /// TOML presets file to use instead of the default location.
    #[arg(long)]
    pub presets_file: Option<PathBuf>,

    /// Load pre-filled variable values from a saved answers file (skips prompts for matched vars).
    #[arg(long)]
    pub answers: Option<PathBuf>,

    /// Save answered variable values to a TOML file after scaffold.
    /// Omit the path to use the default: ~/.config/dex/answers/<template>.toml
    #[arg(long, short = 's', num_args = 0..=1, default_missing_value = "")]
    pub save_answers: Option<String>,
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

    // Capture provenance before the registry borrow ends (needed later to
    // record `.dex/manifest.toml`).
    let origin = entry.origin.clone();

    // Load the template.
    let template = load_template(&entry.source, &args.template)?;

    // Determine default project name from target directory. Slugify to satisfy
    // the common `^[a-z][a-z0-9_]*$` rule: lowercase, hyphens/dots → underscores,
    // strip anything else, ensure a leading alpha.
    let default_project_name = slugify_project_name(
        target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my_project"),
    );

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

        // Skip this variable if its `when` condition evaluates to false.
        if let Some(when_expr) = &spec.when
            && !evaluate_when(when_expr, &variables)
        {
            variables.insert(spec.name.clone(), skipped_default(spec, &effective_default));
            continue;
        }

        // Answers file (highest priority): overrides standards, preset, and defaults.
        if let Some(toml_val) = answers_prefills.get(&spec.name) {
            variables.insert(spec.name.clone(), toml_val_to_minijinja(toml_val));
            continue;
        }

        // Pre-fill: preset or standards value skips the prompt entirely.
        if let Some(val) = prefills.get(&spec.name) {
            variables.insert(spec.name.clone(), prefill_to_value(&spec.var_type, val));
            continue;
        }

        let val = prompt_variable(spec, &effective_default, args.no_prompt)?;
        variables.insert(spec.name.clone(), val);
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

    // Record `.dex/` state (manifest + history + baseline cache) so this
    // project can later be re-synced with `dex update`. Non-fatal: a scaffold
    // that succeeded shouldn't be lost because state-recording hit an error.
    {
        let (source, location, git_ref) = resolve_source_state(&origin, &template);
        if let Err(e) = record_project_state(
            &target,
            &template,
            source,
            location,
            git_ref,
            env!("CARGO_PKG_VERSION"),
            &variables,
        ) {
            output::print_warning(&format!("could not write .dex/ update state: {e}"));
        }
    }

    // If the template suggests skill packs, install them now via the library
    // API (no shell) so scaffold is atomic and survives a missing $PATH.
    // Targets come from the `ai_tools` variable if present, else all four.
    if !template.suggested_skills.is_empty() {
        let targets = resolve_install_targets(&variables);
        println!(
            "  {} Installing suggested skill packs: {}",
            console::style("skills:").cyan().bold(),
            console::style(template.suggested_skills.join(", ")).cyan()
        );
        match install_template_skills(&target, &template.suggested_skills, &targets, true) {
            Ok(n) => {
                let target_names: Vec<&str> = targets.iter().map(|t| t.as_str()).collect();
                println!(
                    "  {} {} skill files installed for: {}\n",
                    console::style("✓").green(),
                    n,
                    console::style(target_names.join(", ")).cyan()
                );
            }
            Err(e) => {
                output::print_warning(&format!(
                    "skill install failed: {e}. Run `dex skills init` manually."
                ));
            }
        }
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

    // Post-scaffold activation hook. Render run/message against the resolved
    // variables so templates can reference user choices (e.g. --targets {{ ai_tools }}).
    if let Some(on_success) = &result.on_success {
        let rendered = render_on_success(on_success, &variables);
        run_on_success(&rendered, &target, args.no_prompt)?;
    }

    Ok(())
}

/// Render `on_success.run` and `on_success.message` through Jinja against the
/// resolved variable map. Rendering failures fall back to the raw string.
fn render_on_success(
    spec: &dex_core::OnSuccessSpec,
    vars: &HashMap<String, minijinja::Value>,
) -> dex_core::OnSuccessSpec {
    let env = minijinja::Environment::new();
    let render = |s: &str| -> String { env.render_str(s, vars).unwrap_or_else(|_| s.to_string()) };
    dex_core::OnSuccessSpec {
        run: spec.run.as_deref().map(render),
        message: spec.message.as_deref().map(render),
    }
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
        println!("  {} {}\n", console::style("next:").cyan().bold(), msg);
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
    /// Where the template really came from — kept alongside `source` because
    /// remotes are resolved to cache directories, which would otherwise lose
    /// the URL needed for `.dex/manifest.toml`.
    origin: TemplateOrigin,
    #[allow(dead_code)]
    description: String,
}

/// Provenance of a discovered template.
#[derive(Clone)]
enum TemplateOrigin {
    Embedded,
    LocalDir(PathBuf),
    Remote(RemoteSource),
}

/// Resolve the manifest source fields for a template from its provenance.
///
/// - Embedded / local directory: `ref` is the template version.
/// - Remote: `ref` is the resolved commit SHA read from the cache clone, so
///   `dex update` can later render the exact same revision. Falls back to the
///   template version if the SHA can't be read (e.g. cache is not a git repo).
fn resolve_source_state(
    origin: &TemplateOrigin,
    template: &Template,
) -> (SourceKind, Option<String>, String) {
    let version = template.meta.version.clone();
    match origin {
        TemplateOrigin::Embedded => (SourceKind::Embedded, None, version),
        TemplateOrigin::LocalDir(path) => (
            SourceKind::Directory,
            Some(path.to_string_lossy().to_string()),
            version,
        ),
        TemplateOrigin::Remote(remote) => {
            let cache = remote_cache_dir().join(&remote.name);
            let git_ref = git_head_sha(&cache).unwrap_or(version);
            (SourceKind::Remote, Some(remote.url.clone()), git_ref)
        }
    }
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
                origin: TemplateOrigin::Embedded,
                description: meta.description,
            },
        );
    }

    // 2. Config-based sources.
    let config = load_dex_config();
    let mut dirs_to_scan: Vec<(TemplateSource, TemplateOrigin)> = Vec::new();

    if let Some(dir) = &config.templates_dir {
        dirs_to_scan.push((
            TemplateSource::Directory(dir.clone()),
            TemplateOrigin::LocalDir(dir.clone()),
        ));
    }

    for remote in &config.remotes {
        match resolve_remote(remote, true) {
            Ok(local) => {
                dirs_to_scan.push((
                    TemplateSource::Directory(local),
                    TemplateOrigin::Remote(remote.clone()),
                ));
            }
            Err(e) => {
                output::print_warning(&format!("could not resolve remote '{}': {e}", remote.name));
            }
        }
    }

    // 3. Extra dir (from extension).
    if let Some(dir) = extra_dir {
        dirs_to_scan.push((
            TemplateSource::Directory(dir.to_path_buf()),
            TemplateOrigin::LocalDir(dir.to_path_buf()),
        ));
    }

    for (source, origin) in &dirs_to_scan {
        if let Ok(metas) = list_templates(source) {
            for meta in metas {
                registry.insert(
                    meta.name.clone(),
                    TemplateEntry {
                        source: source.clone(),
                        origin: origin.clone(),
                        description: meta.description,
                    },
                );
            }
        }
    }

    Ok(registry)
}

/// Coerce a pre-fill string (from a preset or standards file) into a typed
/// minijinja value according to the variable's declared type.
///
/// Preset and standards values always arrive as strings. Without this coercion a
/// `bool` pre-filled as `"false"` becomes a non-empty — and therefore *truthy* —
/// minijinja string, silently enabling the `[[files]]` conditions and `{% if %}`
/// blocks that gate on it. Bools are parsed with the same empty-or-`"true"`
/// convention used for defaults; choice/multi/string values stay strings, which
/// is how the rest of the prompt loop represents them.
fn prefill_to_value(var_type: &VariableType, raw: &str) -> minijinja::Value {
    match var_type {
        VariableType::Bool => minijinja::Value::from(raw.is_empty() || raw == "true"),
        _ => minijinja::Value::from(raw.to_string()),
    }
}

/// Normalize a directory basename into a Python-identifier-compatible slug.
///
/// Agent templates validate `project_name` against `^[a-z][a-z0-9_]*$` because
/// it's used as a Python module path. Without this, `dex init --dir my-agent`
/// would silently produce `src/my-agent/` — a `SyntaxError` at import time.
fn slugify_project_name(name: &str) -> String {
    let lowered: String = name
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '_' => c,
            '-' | '.' | ' ' => '_',
            _ => '_',
        })
        .collect();

    match lowered.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => lowered,
        _ => format!("p_{lowered}"),
    }
}

/// Resolve the install-targets list from a scaffolded project's variables.
/// Honors the `ai_tools` variable if present (CSV string or single token);
/// falls back to all four targets otherwise.
fn resolve_install_targets(variables: &HashMap<String, minijinja::Value>) -> Vec<InstallTarget> {
    let raw = variables
        .get("ai_tools")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    if let Some(s) = raw
        && !s.is_empty()
    {
        let parsed: Vec<InstallTarget> = s
            .split(',')
            .filter_map(|tok| InstallTarget::parse(tok.trim()).ok())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![
        InstallTarget::Claude,
        InstallTarget::Cursor,
        InstallTarget::Copilot,
        InstallTarget::Generic,
    ]
}

fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefill_bool_false_is_not_truthy() {
        // Regression: a preset/standards bool set to "false" must gate OFF, not on.
        assert!(!prefill_to_value(&VariableType::Bool, "false").is_true());
    }

    #[test]
    fn prefill_bool_true_is_truthy() {
        assert!(prefill_to_value(&VariableType::Bool, "true").is_true());
    }

    #[test]
    fn prefill_bool_empty_defaults_to_true() {
        // Matches the empty-or-"true" convention used elsewhere for bool defaults.
        assert!(prefill_to_value(&VariableType::Bool, "").is_true());
    }

    #[test]
    fn prefill_bool_arbitrary_string_is_not_truthy() {
        assert!(!prefill_to_value(&VariableType::Bool, "nope").is_true());
    }

    #[test]
    fn prefill_choice_stays_string() {
        let v = prefill_to_value(&VariableType::Choice, "workspace");
        assert_eq!(v.as_str(), Some("workspace"));
    }

    #[test]
    fn prefill_string_stays_string() {
        let v = prefill_to_value(&VariableType::String, "eng_eus2");
        assert_eq!(v.as_str(), Some("eng_eus2"));
    }
}

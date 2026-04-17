//! Context-Map Generation — emit `.context-map.json` after `dex init`.
//!
//! Produces a machine-readable index of the scaffolded project optimised for
//! LLM consumption. The file tells AI agents what was created, the role of
//! each file, and where to start editing.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::error::DexError;
use crate::scaffold::ScaffoldResult;
use crate::template::Template;

/// A single file entry in the context map.
#[derive(Debug, Serialize)]
pub struct ContextMapFile {
    pub path: String,
    pub role: String,
    pub description: String,
}

/// The full context map written as `.context-map.json`.
#[derive(Debug, Serialize)]
pub struct ContextMap {
    pub schema_version: &'static str,
    pub generated_by: String,
    pub template: String,
    pub scaffolded_at: String,
    pub variables: HashMap<String, String>,
    pub files: Vec<ContextMapFile>,
    pub entry_points: Vec<String>,
    pub tasks: Vec<String>,
    pub traits: Vec<String>,
}

/// Write `.context-map.json` to `dir` after a scaffold operation completes.
///
/// Non-fatal: callers should print a warning on error rather than aborting.
pub fn write_context_map(
    result: &ScaffoldResult,
    template: &Template,
    dir: &Path,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<(), DexError> {
    let map = build_context_map(result, template, dir, variables);
    let json = serde_json::to_string_pretty(&map).map_err(|e| DexError::Io {
        path: dir.join(".context-map.json"),
        source: std::io::Error::other(e),
    })?;
    let out_path = dir.join(".context-map.json");
    std::fs::write(&out_path, json).map_err(|source| DexError::Io {
        path: out_path,
        source,
    })
}

/// Build a [`ContextMap`] from scaffold output without writing it.
///
/// Exposed for testing.
pub fn build_context_map(
    result: &ScaffoldResult,
    template: &Template,
    dir: &Path,
    variables: &HashMap<String, minijinja::Value>,
) -> ContextMap {
    let generated_by = format!("dex {}", env!("CARGO_PKG_VERSION"));

    // Convert minijinja variable values to plain strings for the JSON output.
    let vars: HashMap<String, String> = variables
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    // Build file entries, annotating roles from template file rules where available.
    let files: Vec<ContextMapFile> = result
        .files_created
        .iter()
        .map(|p| file_entry(p, template))
        .collect();

    // Collect entry points: files with role == "entry_point".
    let entry_points: Vec<String> = files
        .iter()
        .filter(|f| f.role == "entry_point")
        .map(|f| f.path.clone())
        .collect();

    // Read task names from the scaffolded dex.toml. Best-effort; empty list is fine.
    let tasks: Vec<String> = match crate::config::load_project_config(&dir.join("dex.toml")) {
        Ok(cfg) => {
            let mut names: Vec<String> = cfg.tasks.into_keys().collect();
            names.sort();
            names
        }
        Err(_) => Vec::new(),
    };

    ContextMap {
        schema_version: "1",
        generated_by,
        template: template.meta.name.clone(),
        scaffolded_at: now_iso8601(),
        variables: vars,
        files,
        entry_points,
        tasks,
        traits: Vec::new(),
    }
}

/// Determine the role and description for a file based on template file rules.
fn file_entry(rel_path: &Path, template: &Template) -> ContextMapFile {
    let path_str = rel_path.to_string_lossy().into_owned();

    // Look for a matching file rule with context annotations.
    for rule in &template.file_rules {
        let rule_src = Path::new(&rule.src);
        if (rel_path.starts_with(rule_src) || rel_path == rule_src)
            && let (Some(role), Some(desc)) = (&rule.context_role, &rule.context_description)
        {
            return ContextMapFile {
                path: path_str,
                role: role.clone(),
                description: desc.clone(),
            };
        }
    }

    // Infer role from path when no annotation is present.
    let (role, description) = infer_role(rel_path);
    ContextMapFile {
        path: path_str,
        role: role.to_string(),
        description: description.to_string(),
    }
}

/// Infer a role and short description from a file path.
fn infer_role(path: &Path) -> (&'static str, &'static str) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    match name {
        "dex.toml" => (
            "config",
            "dex project config. Defines tasks and pass-through commands.",
        ),
        "databricks.yml" => ("bundle_config", "Databricks Asset Bundle definition."),
        "pyproject.toml" => ("config", "Python project metadata and dependencies."),
        "README.md" => ("docs", "Project README."),
        "CLAUDE.md" => ("ai_context", "Project instructions for Claude Code."),
        _ if is_entry_point(path) => (
            "entry_point",
            "Main entry point. Edit this to implement your logic.",
        ),
        _ if path.starts_with(Path::new("tests")) => ("test", "Test file."),
        _ if path.starts_with(Path::new("evals")) => ("eval", "Evaluation case."),
        _ if path.starts_with(Path::new("resources")) => {
            ("bundle_resource", "DAB resource definition.")
        }
        _ if path.starts_with(Path::new("notebooks")) => ("notebook", "Databricks notebook."),
        _ if path.starts_with(Path::new("src")) => ("source", "Source file."),
        _ => ("other", ""),
    }
}

/// Heuristically detect entry-point files.
fn is_entry_point(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    // Check by name
    if matches!(
        name,
        "main.py" | "agent.py" | "app.py" | "main.rs" | "index.ts" | "App.tsx"
    ) {
        return true;
    }
    // Next.js: src/app/page.tsx is the root page entry point
    if name == "page.tsx"
        && path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some("app")
    {
        return true;
    }
    false
}

/// Format the current UTC time as an ISO 8601 string without external crates.
fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (year, month, day, h, m, s) = unix_secs_to_parts(secs);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Decompose a Unix timestamp into (year, month, day, hour, minute, second) UTC.
fn unix_secs_to_parts(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let h = ((secs % 86400) / 3600) as u32;
    let m = ((secs % 3600) / 60) as u32;
    let s = (secs % 60) as u32;

    let mut days = secs / 86400;
    let mut year = 1970u32;
    loop {
        let diy = days_in_year(year);
        if days < diy {
            break;
        }
        days -= diy;
        year += 1;
    }

    let leap = is_leap(year);
    let month_lens: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &ml in &month_lens {
        if days < ml {
            break;
        }
        days -= ml;
        month += 1;
    }

    (year, month, days as u32 + 1, h, m, s)
}

fn days_in_year(year: u32) -> u64 {
    if is_leap(year) { 366 } else { 365 }
}

fn is_leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_epoch() {
        assert_eq!(unix_secs_to_parts(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn iso8601_known_date() {
        // 2026-04-06 00:00:00 UTC = 1775433600
        let (y, mo, d, h, m, s) = unix_secs_to_parts(1_775_433_600);
        assert_eq!((y, mo, d, h, m, s), (2026, 4, 6, 0, 0, 0));
    }

    #[test]
    fn build_context_map_populates_tasks() {
        use crate::template::TemplateMeta;

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("dex.toml"),
            "[project]\nname = \"my-app\"\ntemplate = \"databricks-app-streamlit\"\n\
             [tasks.deploy]\ncommand = \"databricks bundle deploy\"\n\
             [tasks.dev]\ncommand = \"streamlit run app/app.py\"\n",
        )
        .unwrap();

        let result = ScaffoldResult {
            files_created: vec![std::path::PathBuf::from("dex.toml")],
            directories_created: Vec::new(),
            on_success: None,
        };

        let template = Template {
            meta: TemplateMeta {
                name: "databricks-app-streamlit".into(),
                description: "test".into(),
                version: "0.1.0".into(),
                min_dex_version: None,
            },
            variables: vec![],
            file_rules: vec![],
            files: std::collections::HashMap::new(),
            suggested_skills: vec![],
            on_success: None,
        };

        let vars = HashMap::new();
        let map = build_context_map(&result, &template, dir.path(), &vars);

        assert_eq!(map.tasks, vec!["deploy", "dev"]); // sorted
    }

    #[test]
    fn build_context_map_produces_files() {
        use crate::template::TemplateMeta;

        let files_created = vec![
            std::path::PathBuf::from("dex.toml"),
            std::path::PathBuf::from("src/main.py"),
            std::path::PathBuf::from("README.md"),
        ];

        let result = ScaffoldResult {
            files_created,
            directories_created: Vec::new(),
            on_success: None,
        };

        let template = Template {
            meta: TemplateMeta {
                name: "test".into(),
                description: "Test".into(),
                version: "0.1.0".into(),
                min_dex_version: None,
            },
            variables: vec![],
            file_rules: vec![],
            files: std::collections::HashMap::new(),
            suggested_skills: vec![],
            on_success: None,
        };

        let vars = HashMap::new();
        let dir = Path::new("/tmp");
        let map = build_context_map(&result, &template, dir, &vars);

        assert_eq!(map.schema_version, "1");
        assert_eq!(map.template, "test");
        assert_eq!(map.files.len(), 3);

        let dex_toml = map.files.iter().find(|f| f.path == "dex.toml").unwrap();
        assert_eq!(dex_toml.role, "config");

        let main_py = map.files.iter().find(|f| f.path == "src/main.py").unwrap();
        assert_eq!(main_py.role, "entry_point");
    }
}

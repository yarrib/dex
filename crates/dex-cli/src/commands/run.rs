//! `dex run <task>` — run a task defined in dex.toml.

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use clap::Args;
use console::style;

use dex_core::DexError;
use dex_core::config::{ProjectConfig, TaskSpec, load_project_config};

use crate::output;

#[derive(Args)]
pub struct RunArgs {
    /// Task name to run (from [tasks.*] in dex.toml).
    pub task: String,

    /// Extra arguments forwarded to the task command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

pub fn run(args: RunArgs) -> Result<(), DexError> {
    let config = load_project_config(&PathBuf::from("dex.toml")).map_err(|e| {
        DexError::Config(dex_core::error::ConfigError::Invalid(format!(
            "could not load dex.toml: {e}"
        )))
    })?;

    if !config.tasks.contains_key(args.task.as_str()) {
        let available: Vec<&str> = config.tasks.keys().map(String::as_str).collect();
        return Err(DexError::Config(dex_core::error::ConfigError::Invalid(
            if available.is_empty() {
                format!("unknown task '{}'. No tasks defined in dex.toml.", args.task)
            } else {
                format!(
                    "unknown task '{}'. Available: {}",
                    args.task,
                    available.join(", ")
                )
            },
        )));
    }

    let mut visited: HashSet<String> = HashSet::new();
    let order = resolve_order(&config, &args.task, &mut visited, &mut vec![])?;

    for task_name in &order {
        let spec = &config.tasks[task_name];
        run_task(task_name, spec, if task_name == &args.task { &args.extra } else { &[] })?;
    }

    Ok(())
}

/// Build an ordered run list for `task_name` respecting `depends_on`.
/// Returns tasks in execution order (dependencies first).
fn resolve_order(
    config: &ProjectConfig,
    task_name: &str,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> Result<Vec<String>, DexError> {
    if stack.contains(&task_name.to_string()) {
        return Err(DexError::Config(dex_core::error::ConfigError::Invalid(
            format!("cycle detected in depends_on for task '{task_name}'"),
        )));
    }

    if visited.contains(task_name) {
        return Ok(vec![]);
    }

    let spec = config.tasks.get(task_name).ok_or_else(|| {
        DexError::Config(dex_core::error::ConfigError::Invalid(format!(
            "depends_on references unknown task '{task_name}'"
        )))
    })?;

    stack.push(task_name.to_string());

    let mut order = vec![];
    for dep in &spec.depends_on {
        order.extend(resolve_order(config, dep, visited, stack)?);
    }
    order.push(task_name.to_string());

    stack.pop();
    visited.insert(task_name.to_string());

    Ok(order)
}

/// Execute a single task, appending any extra args to its command.
fn run_task(name: &str, spec: &TaskSpec, extra: &[String]) -> Result<(), DexError> {
    let mut full_command = spec.command.clone();
    if !extra.is_empty() {
        full_command.push(' ');
        full_command.push_str(&extra.join(" "));
    }

    println!("{} {}", style("$").dim(), style(&full_command).bold());

    #[cfg(windows)]
    let status = Command::new("cmd")
        .args(["/C", &full_command])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|source| DexError::Io {
            path: PathBuf::from("cmd"),
            source,
        })?;

    #[cfg(not(windows))]
    let status = Command::new("sh")
        .args(["-c", &full_command])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|source| DexError::Io {
            path: PathBuf::from("sh"),
            source,
        })?;

    if !status.success() {
        output::print_error(&format!("task '{name}' failed (exit {})", status.code().unwrap_or(1)));
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dex_core::config::{ProjectConfig, ProjectMeta, TaskSpec};

    use super::resolve_order;

    fn make_config(tasks: Vec<(&str, &str, Vec<&str>)>) -> ProjectConfig {
        let mut map = HashMap::new();
        for (name, cmd, deps) in tasks {
            map.insert(
                name.to_string(),
                TaskSpec {
                    command: cmd.to_string(),
                    description: None,
                    depends_on: deps.iter().map(|s| s.to_string()).collect(),
                },
            );
        }
        ProjectConfig {
            project: ProjectMeta {
                name: "test".into(),
                description: None,
                template: None,
            },
            tasks: map,
            profiles: HashMap::new(),
            passthrough: HashMap::new(),
        }
    }

    #[test]
    fn test_no_deps() {
        let config = make_config(vec![("hello", "echo hello", vec![])]);
        let order = resolve_order(&config, "hello", &mut Default::default(), &mut vec![]).unwrap();
        assert_eq!(order, vec!["hello"]);
    }

    #[test]
    fn test_depends_on_ordering() {
        let config = make_config(vec![
            ("build", "cargo build", vec![]),
            ("test", "cargo test", vec!["build"]),
        ]);
        let order = resolve_order(&config, "test", &mut Default::default(), &mut vec![]).unwrap();
        assert_eq!(order, vec!["build", "test"]);
    }

    #[test]
    fn test_cycle_detection() {
        let config = make_config(vec![
            ("a", "echo a", vec!["b"]),
            ("b", "echo b", vec!["a"]),
        ]);
        let result = resolve_order(&config, "a", &mut Default::default(), &mut vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_unknown_dep_errors() {
        let config = make_config(vec![("deploy", "echo deploy", vec!["build"])]);
        let result = resolve_order(&config, "deploy", &mut Default::default(), &mut vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_dedup_shared_deps() {
        let config = make_config(vec![
            ("compile", "echo compile", vec![]),
            ("lint", "echo lint", vec!["compile"]),
            ("test", "echo test", vec!["compile"]),
            ("ci", "echo ci", vec!["lint", "test"]),
        ]);
        let order = resolve_order(&config, "ci", &mut Default::default(), &mut vec![]).unwrap();
        // compile must appear once and before lint/test
        assert_eq!(order.iter().filter(|x| x.as_str() == "compile").count(), 1);
        let compile_pos = order.iter().position(|x| x == "compile").unwrap();
        let lint_pos = order.iter().position(|x| x == "lint").unwrap();
        let test_pos = order.iter().position(|x| x == "test").unwrap();
        assert!(compile_pos < lint_pos);
        assert!(compile_pos < test_pos);
    }
}

//! Agent project scaffolding for `dex agent new`.
//!
//! This module handles the deterministic phase of agent project generation:
//! parsing Q&A answers, rendering templates, and producing the project structure.
//! The generative phase (Claude API) is handled in the Python layer.

pub mod spec;

pub use spec::{AgentAnswers, AgentDeployTarget, AgentTrigger};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::DexError;
use crate::template::engine::TemplateEngine;

/// Result of agent project scaffolding (deterministic phase).
#[derive(Debug)]
pub struct AgentScaffoldResult {
    pub project_dir: PathBuf,
    pub files_created: Vec<PathBuf>,
    pub system_prompt: String,
    pub claude_md: String,
}

/// Scaffold an agent project from Q&A answers.
///
/// This generates the full project structure deterministically. The generative
/// phase (fleshing out agent.py, tool stubs, etc.) happens in the Python layer
/// via the Anthropic API or Claude Code CLI.
pub fn scaffold_agent(
    answers: &AgentAnswers,
    target_dir: &Path,
) -> Result<AgentScaffoldResult, DexError> {
    let engine = TemplateEngine::new();
    let project_name = slugify(&answers.name);
    let project_dir = target_dir.join(&project_name);

    // Build template context from answers.
    let context = answers_to_context(answers, &project_name);
    let ctx_value = minijinja::Value::from_serialize(&context);

    if !project_dir.exists() {
        std::fs::create_dir_all(&project_dir).map_err(|source| DexError::Io {
            path: project_dir.clone(),
            source,
        })?;
    }

    let mut files_created = Vec::new();

    // Generate each file from the embedded agent template.
    let file_specs = agent_file_specs(answers);

    for (rel_path_template, content_template) in &file_specs {
        let rel_path_str = engine.render_path(rel_path_template, &ctx_value)?;
        let rel_path = PathBuf::from(&rel_path_str);
        let dest = project_dir.join(&rel_path);

        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
        }

        let rendered = engine.render_string(content_template, &ctx_value)?;
        std::fs::write(&dest, &rendered).map_err(|source| DexError::Io {
            path: dest.clone(),
            source,
        })?;
        files_created.push(rel_path);
    }

    // Generate system prompt from answers.
    let system_prompt = generate_system_prompt(answers, &engine, &ctx_value)?;
    let prompt_path = project_dir
        .join("src")
        .join(&project_name)
        .join("prompts")
        .join("system.md");
    if let Some(parent) = prompt_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(&prompt_path, &system_prompt).map_err(|source| DexError::Io {
        path: prompt_path,
        source,
    })?;
    files_created.push(PathBuf::from(format!(
        "src/{}/prompts/system.md",
        project_name
    )));

    // Generate CLAUDE.md.
    let claude_md = generate_claude_md(answers, &engine, &ctx_value)?;
    let claude_path = project_dir.join("CLAUDE.md");
    std::fs::write(&claude_path, &claude_md).map_err(|source| DexError::Io {
        path: claude_path,
        source,
    })?;
    files_created.push(PathBuf::from("CLAUDE.md"));

    Ok(AgentScaffoldResult {
        project_dir,
        files_created,
        system_prompt,
        claude_md,
    })
}

/// Convert Q&A answers to a template context map.
fn answers_to_context(answers: &AgentAnswers, project_name: &str) -> HashMap<String, String> {
    let mut ctx = HashMap::new();
    ctx.insert("project_name".into(), project_name.into());
    ctx.insert("agent_name".into(), answers.name.clone());
    ctx.insert("description".into(), answers.description.clone());
    ctx.insert(
        "trigger".into(),
        format!("{:?}", answers.trigger).to_lowercase(),
    );
    ctx.insert("success_criteria".into(), answers.success_criteria.clone());
    ctx.insert("reads".into(), answers.reads.clone());
    ctx.insert("writes".into(), answers.writes.clone());
    ctx.insert(
        "deploy_target".into(),
        format!("{:?}", answers.deploy_target).to_lowercase(),
    );
    ctx.insert("autonomous".into(), answers.autonomous.to_string());
    ctx.insert("example_input".into(), answers.example_input.clone());
    ctx.insert("example_output".into(), answers.example_output.clone());
    ctx.insert("bad_output".into(), answers.bad_output.clone());
    ctx
}

/// Slugify a name for use as a Python module/package name.
///
/// Converts to lowercase, replaces hyphens and any non-alphanumeric characters
/// with underscores. Hyphens are explicitly replaced because hyphenated names
/// are invalid as Python identifiers (e.g. `import table-anomaly` is a syntax error).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Generate the list of files to create with their template content.
fn agent_file_specs(answers: &AgentAnswers) -> Vec<(String, String)> {
    let deploy_resource = match answers.deploy_target {
        AgentDeployTarget::Job => "{{ project_name }}_job.yml",
        AgentDeployTarget::ServingEndpoint => "{{ project_name }}_serving.yml",
        AgentDeployTarget::Interactive => "{{ project_name }}_job.yml",
    };

    vec![
        // pyproject.toml
        ("pyproject.toml".into(), PYPROJECT_TEMPLATE.into()),
        // databricks.yml
        ("databricks.yml".into(), DATABRICKS_YML_TEMPLATE.into()),
        // DAB resource
        (
            format!("resources/{deploy_resource}"),
            match answers.deploy_target {
                AgentDeployTarget::Job => JOB_RESOURCE_TEMPLATE.into(),
                AgentDeployTarget::ServingEndpoint => SERVING_RESOURCE_TEMPLATE.into(),
                AgentDeployTarget::Interactive => JOB_RESOURCE_TEMPLATE.into(),
            },
        ),
        // Agent source
        ("src/{{ project_name }}/__init__.py".into(), "".into()),
        (
            "src/{{ project_name }}/agent.py".into(),
            AGENT_PY_TEMPLATE.into(),
        ),
        (
            "src/{{ project_name }}/tools/__init__.py".into(),
            TOOLS_INIT_TEMPLATE.into(),
        ),
        (
            "src/{{ project_name }}/tools/example_tool.py".into(),
            EXAMPLE_TOOL_TEMPLATE.into(),
        ),
        (
            "src/{{ project_name }}/tracing.py".into(),
            TRACING_TEMPLATE.into(),
        ),
        // Evals
        ("evals/runner.py".into(), EVAL_RUNNER_TEMPLATE.into()),
        ("evals/cases/example.json".into(), EVAL_CASE_TEMPLATE.into()),
        // Tests
        ("tests/__init__.py".into(), "".into()),
        ("tests/test_agent.py".into(), TEST_AGENT_TEMPLATE.into()),
        // .env.example
        (".env.example".into(), ENV_EXAMPLE_TEMPLATE.into()),
    ]
}

fn generate_system_prompt(
    _answers: &AgentAnswers,
    engine: &TemplateEngine,
    ctx: &minijinja::Value,
) -> Result<String, DexError> {
    engine.render_string(SYSTEM_PROMPT_TEMPLATE, ctx)
}

fn generate_claude_md(
    _answers: &AgentAnswers,
    engine: &TemplateEngine,
    ctx: &minijinja::Value,
) -> Result<String, DexError> {
    engine.render_string(CLAUDE_MD_TEMPLATE, ctx)
}

// --- Embedded templates ---

const PYPROJECT_TEMPLATE: &str = r#"[project]
name = "{{ project_name }}"
version = "0.1.0"
description = "{{ description }}"
requires-python = ">=3.10"
dependencies = [
    "anthropic",
    "mlflow",
    "databricks-sdk",
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.ruff]
target-version = "py310"
line-length = 99

[tool.pytest.ini_options]
testpaths = ["tests"]
"#;

const DATABRICKS_YML_TEMPLATE: &str = r#"bundle:
  name: {{ project_name }}

include:
  - resources/*.yml

workspace:
  host: ${var.workspace_url}

variables:
  workspace_url:
    description: Databricks workspace URL

targets:
  dev:
    mode: development
    default: true
    variables:
      workspace_url: ""

  staging:
    variables:
      workspace_url: ""

  prod:
    variables:
      workspace_url: ""
"#;

const JOB_RESOURCE_TEMPLATE: &str = r#"resources:
  jobs:
    {{ project_name }}:
      name: {{ project_name }}
      {% if trigger == "schedule" %}schedule:
        quartz_cron_expression: "0 0 * * * ?"
        timezone_id: UTC{% endif %}
      tasks:
        - task_key: run_agent
          python_wheel_task:
            package_name: {{ project_name }}
            entry_point: agent
          libraries:
            - whl: ../dist/*.whl
"#;

const SERVING_RESOURCE_TEMPLATE: &str = r#"resources:
  model_serving_endpoints:
    {{ project_name }}:
      name: {{ project_name }}
      config:
        served_entities:
          - entity_name: {{ project_name }}
            entity_version: "1"
            workload_size: Small
            scale_to_zero_enabled: true
"#;

const AGENT_PY_TEMPLATE: &str = r#""""{{ agent_name }} — agent definition and main loop."""

import logging
from pathlib import Path

from {{ project_name }}.tracing import setup_tracing
from {{ project_name }}.tools import discover_tools

logger = logging.getLogger(__name__)

SYSTEM_PROMPT = (Path(__file__).parent / "prompts" / "system.md").read_text()


def run(input_data: str | None = None) -> str:
    """Run the agent.

    This is the entry point. The deterministic scaffold sets up structure;
    Claude Code should flesh out the agent logic here.

    Args:
        input_data: Input to the agent. None for scheduled/event-triggered agents.

    Returns:
        Agent output or status message.
    """
    setup_tracing("{{ project_name }}")
    tools = discover_tools()

    logger.info("agent_start", extra={"input": input_data, "tools": list(tools.keys())})

    # TODO: Implement agent logic here.
    # The generative phase (Claude) will fill this in based on:
    #   Description: {{ description }}
    #   Trigger: {{ trigger }}
    #   Success: {{ success_criteria }}
    #   Reads: {{ reads }}
    #   Writes: {{ writes }}

    result = "NOT_IMPLEMENTED"

    logger.info("agent_complete", extra={"result": result})
    return result


def main():
    """CLI entry point for local development."""
    import json
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    result = run()
    print(json.dumps({"result": result}, indent=2))


if __name__ == "__main__":
    main()
"#;

const TOOLS_INIT_TEMPLATE: &str = r#""""Tool discovery and registration."""

import importlib
import pkgutil
from dataclasses import dataclass
from typing import Any, Callable


@dataclass
class ToolResult:
    """Standard return type for all tools."""
    success: bool
    data: Any
    error: str | None = None


def discover_tools() -> dict[str, Callable]:
    """Auto-discover tools from the tools/ directory."""
    tools = {}
    package = importlib.import_module(__package__)

    for _, module_name, _ in pkgutil.iter_modules(package.__path__):
        if module_name.startswith("_"):
            continue
        module = importlib.import_module(f"{__package__}.{module_name}")
        # Register any callable with a __tool__ attribute or whose name doesn't start with _
        for attr_name in dir(module):
            if attr_name.startswith("_"):
                continue
            attr = getattr(module, attr_name)
            if callable(attr) and hasattr(attr, "__doc__") and attr.__doc__:
                tools[attr_name] = attr

    return tools
"#;

const EXAMPLE_TOOL_TEMPLATE: &str = r#""""Example tool — replace with real tools based on agent requirements.

Reads: {{ reads }}
Writes: {{ writes }}
"""

from {{ project_name }}.tools import ToolResult


def example_tool(param: str) -> ToolResult:
    """Example tool that demonstrates the tool interface.

    TODO: Replace this with a real tool. The generative phase (Claude)
    will create tool stubs based on what the agent needs to read/write.
    """
    return ToolResult(success=True, data=f"processed: {param}")
"#;

const TRACING_TEMPLATE: &str = r#""""MLflow tracing setup — always included, always wired in."""

import mlflow


def setup_tracing(experiment_name: str) -> None:
    """Configure MLflow tracing for agent runs.

    Every agent run is traced to an MLflow experiment. Traces include:
    input, output, tool calls, latency, and errors.
    """
    mlflow.set_experiment(experiment_name)
    mlflow.autolog()
"#;

const EVAL_RUNNER_TEMPLATE: &str = r#""""Eval harness — loads cases, runs the agent, logs results to MLflow."""

import json
import logging
from pathlib import Path

import mlflow

from {{ project_name }}.agent import run

logger = logging.getLogger(__name__)


def load_cases(cases_dir: Path | None = None) -> list[dict]:
    """Load eval cases from JSON files."""
    if cases_dir is None:
        cases_dir = Path(__file__).parent / "cases"
    cases = []
    for path in sorted(cases_dir.glob("*.json")):
        with open(path) as f:
            cases.append(json.load(f))
    return cases


def run_evals(cases_dir: Path | None = None) -> dict:
    """Run all eval cases and log results to MLflow."""
    cases = load_cases(cases_dir)
    results = {"total": len(cases), "passed": 0, "failed": 0}

    with mlflow.start_run(run_name="eval"):
        for case in cases:
            case_id = case.get("id", "unknown")
            logger.info(f"Running eval case: {case_id}")

            try:
                output = run(case.get("input"))
                # TODO: Add assertion logic based on expected_behavior
                results["passed"] += 1
                mlflow.log_metric(f"case_{case_id}_pass", 1)
            except Exception as e:
                results["failed"] += 1
                mlflow.log_metric(f"case_{case_id}_pass", 0)
                logger.error(f"Case {case_id} failed: {e}")

        mlflow.log_metrics({
            "eval_total": results["total"],
            "eval_passed": results["passed"],
            "eval_failed": results["failed"],
        })

    return results


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    results = run_evals()
    print(json.dumps(results, indent=2))
"#;

const EVAL_CASE_TEMPLATE: &str = r#"{
    "id": "example-01",
    "description": "Basic happy path",
    "input": "{{ example_input }}",
    "expected_behavior": "{{ example_output }}",
    "should_not": "{{ bad_output }}"
}
"#;

const TEST_AGENT_TEMPLATE: &str = r#""""Tests for {{ project_name }}."""

from {{ project_name }}.agent import run
from {{ project_name }}.tools import discover_tools


def test_agent_runs():
    """Smoke test — agent runs without error."""
    result = run()
    assert result is not None


def test_tools_discovered():
    """Tools are discoverable from the tools/ directory."""
    tools = discover_tools()
    assert isinstance(tools, dict)
"#;

const ENV_EXAMPLE_TEMPLATE: &str = r#"# Anthropic API key for agent LLM calls
ANTHROPIC_API_KEY=

# Databricks workspace config (or use ~/.databrickscfg)
DATABRICKS_HOST=
DATABRICKS_TOKEN=
"#;

const SYSTEM_PROMPT_TEMPLATE: &str = r#"# {{ agent_name }}

{{ description }}

## Behavior

- **Trigger:** {{ trigger }}
- **Success criteria:** {{ success_criteria }}
{% if autonomous %}- **Mode:** Autonomous — act without confirmation{% else %}- **Mode:** Confirm before taking actions{% endif %}

## Data Access

- **Reads:** {{ reads }}
- **Writes:** {{ writes }}

## Constraints

- Never produce output that looks like: {{ bad_output }}
- Always trace operations via MLflow
- Log structured JSON, never print statements

## Example

**Input:** {{ example_input }}
**Expected behavior:** {{ example_output }}
"#;

const CLAUDE_MD_TEMPLATE: &str = r#"# CLAUDE.md — {{ agent_name }}

## What This Agent Does

{{ description }}

## Project Structure

```
{{ project_name }}/
├── src/{{ project_name }}/
│   ├── agent.py            # Agent definition and main loop
│   ├── tools/              # Tool implementations (auto-discovered)
│   ├── prompts/system.md   # System prompt
│   └── tracing.py          # MLflow tracing (do not remove)
├── evals/                  # Eval cases and runner
├── resources/              # DAB resource definitions
├── tests/
├── databricks.yml          # DAB root config
└── pyproject.toml
```

## Conventions

- All tools return `ToolResult(success, data, error)`.
- Tools are auto-discovered from `tools/` — any public function with a docstring.
- The system prompt in `prompts/system.md` is loaded at startup.
- MLflow tracing is always on. Do not remove or disable `tracing.py`.
- Structured JSON logging only. No `print()` statements.
- This project is a Databricks Asset Bundle — deploy with `dex deploy`.

## Build & Test

```bash
uv sync
uv run pytest
uv run python evals/runner.py
```

## Deploy

```bash
dex deploy            # deploy to dev (default target)
dex deploy staging    # deploy to staging
```
"#;

#[cfg(test)]
mod tests {
    use super::spec::*;
    use super::*;

    fn make_answers() -> AgentAnswers {
        AgentAnswers {
            name: "table-anomaly-monitor".into(),
            description: "Monitors a Delta table for anomalies".into(),
            trigger: AgentTrigger::Schedule,
            success_criteria: "Slack alert sent".into(),
            reads: "Unity Catalog table: main.monitoring.events".into(),
            writes: "Nothing, read-only plus Slack".into(),
            handoff: false,
            autonomous: true,
            example_input: "Row count drops 80%".into(),
            example_output: "Send alert with table name and counts".into(),
            bad_output: "Alerting on normal variance".into(),
            deploy_target: AgentDeployTarget::Job,
        }
    }

    #[test]
    fn slugify_names() {
        assert_eq!(slugify("My Cool Agent"), "my_cool_agent");
        assert_eq!(slugify("table-anomaly-monitor"), "table_anomaly_monitor");
        assert_eq!(slugify("my.package"), "my_package");
        assert_eq!(slugify("__leading"), "leading");
    }

    #[test]
    fn scaffold_agent_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let answers = make_answers();
        let result = scaffold_agent(&answers, dir.path()).unwrap();

        assert!(result.project_dir.exists());
        assert!(result.files_created.len() > 10);
        assert!(result.system_prompt.contains("anomalies"));
        assert!(result.claude_md.contains("table_anomaly_monitor"));

        // Check key files exist
        assert!(result.project_dir.join("pyproject.toml").exists());
        assert!(result.project_dir.join("databricks.yml").exists());
        assert!(result.project_dir.join("CLAUDE.md").exists());
        assert!(result
            .project_dir
            .join("src/table_anomaly_monitor/agent.py")
            .exists());
        assert!(result
            .project_dir
            .join("src/table_anomaly_monitor/tracing.py")
            .exists());
        assert!(result.project_dir.join("evals/cases/example.json").exists());
    }
}

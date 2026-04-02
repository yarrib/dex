//! `dex mcp serve` — JSON-RPC 2.0 MCP server over stdio.
//!
//! Exposes dex operations to MCP clients (Claude Desktop, Claude Code, etc.)
//! via the Model Context Protocol over stdin/stdout.
//!
//! Tools provided:
//! - `list_templates` — list all available dex templates
//! - `get_template_variables` — return variable specs for a named template
//! - `scaffold_project` — scaffold a project from a template into a directory

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde_json::{Value, json};

use dex_core::agent::{AgentAnswers, AgentDeployTarget, AgentTrigger, scaffold_agent};
use dex_core::error::ConfigError;
use dex_core::template::TemplateSource;
use dex_core::template::registry::{list_templates, load_template};
use dex_core::template::variables::VariableType;
use dex_core::{DexError, scaffold};

#[derive(Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub cmd: McpCommand,
}

#[derive(Subcommand)]
pub enum McpCommand {
    /// Start the MCP server on stdio (JSON-RPC 2.0).
    Serve,
}

pub fn run(args: McpArgs) -> Result<(), DexError> {
    match args.cmd {
        McpCommand::Serve => serve(),
    }
}

fn serve() -> Result<(), DexError> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {e}") }
                });
                writeln!(out, "{response}").ok();
                out.flush().ok();
                continue;
            }
        };

        // Notifications have no id — no response.
        let Some(id) = msg.get("id").cloned() else {
            continue;
        };

        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        let response = match method {
            "initialize" => handle_initialize(&id),
            "tools/list" => handle_tools_list(&id),
            "tools/call" => handle_tools_call(&id, &params),
            "ping" => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
            _ => error_response(&id, -32601, &format!("Method not found: {method}")),
        };

        writeln!(out, "{response}").ok();
        out.flush().ok();
    }

    Ok(())
}

// --- Protocol handlers ---

fn handle_initialize(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "dex",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    })
}

fn handle_tools_list(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "list_templates",
                    "description": "List all available dex project templates with their names and descriptions.",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "get_template_variables",
                    "description": "Return the variable specifications for a named template. Call this before scaffold_project to know which variables to supply.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "template": {
                                "type": "string",
                                "description": "Template name (e.g. 'dabs-package', 'default')"
                            }
                        },
                        "required": ["template"]
                    }
                },
                {
                    "name": "scaffold_project",
                    "description": "Scaffold a new project from a dex template. Creates files in the target directory.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "template": {
                                "type": "string",
                                "description": "Template name (e.g. 'dabs-package', 'default')"
                            },
                            "dir": {
                                "type": "string",
                                "description": "Target directory path. Created if it does not exist."
                            },
                            "variables": {
                                "type": "object",
                                "description": "Variable values keyed by name. Use get_template_variables to see what is available. Missing variables use their defaults."
                            }
                        },
                        "required": ["template", "dir"]
                    }
                },
                {
                    "name": "scaffold_agent",
                    "description": "Scaffold a new Databricks AI agent project. Creates a full project structure with agent.py, tools, evals, DAB resources, a system prompt, and CLAUDE.md. Call this when the user wants to create a new AI agent for Databricks.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Agent name (e.g. 'table-anomaly-monitor'). Used as the Python package name."
                            },
                            "dir": {
                                "type": "string",
                                "description": "Parent directory where the agent project folder will be created."
                            },
                            "description": {
                                "type": "string",
                                "description": "One-sentence description of what the agent does."
                            },
                            "trigger": {
                                "type": "string",
                                "enum": ["user_request", "schedule", "event", "upstream_system"],
                                "description": "What triggers the agent. Default: user_request."
                            },
                            "success_criteria": {
                                "type": "string",
                                "description": "What success looks like (e.g. 'Slack alert sent with anomaly details')."
                            },
                            "reads": {
                                "type": "string",
                                "description": "What data sources the agent reads (e.g. 'Unity Catalog table: main.monitoring.events')."
                            },
                            "writes": {
                                "type": "string",
                                "description": "What the agent writes or changes (e.g. 'Slack channel #alerts')."
                            },
                            "handoff": {
                                "type": "boolean",
                                "description": "Whether the agent hands off to a human or another agent. Default: false."
                            },
                            "autonomous": {
                                "type": "boolean",
                                "description": "Whether the agent acts autonomously without confirmation. Default: true."
                            },
                            "example_input": {
                                "type": "string",
                                "description": "Example input to the agent."
                            },
                            "example_output": {
                                "type": "string",
                                "description": "Expected output for the example input."
                            },
                            "bad_output": {
                                "type": "string",
                                "description": "What a bad or dangerous output looks like."
                            },
                            "deploy_target": {
                                "type": "string",
                                "enum": ["job", "serving_endpoint", "interactive"],
                                "description": "How the agent is deployed on Databricks. Default: job."
                            }
                        },
                        "required": ["name", "dir"]
                    }
                }
            ]
        }
    })
}

fn handle_tools_call(id: &Value, params: &Value) -> Value {
    let Some(name) = params.get("name").and_then(|n| n.as_str()) else {
        return error_response(id, -32602, "Missing required parameter: name");
    };
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let result = match name {
        "list_templates" => tool_list_templates(),
        "get_template_variables" => tool_get_template_variables(&args),
        "scaffold_project" => tool_scaffold_project(&args),
        "scaffold_agent" => tool_scaffold_agent(&args),
        _ => return error_response(id, -32602, &format!("Unknown tool: {name}")),
    };

    match result {
        Ok(text) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "content": [{ "type": "text", "text": text }] }
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "isError": true,
                "content": [{ "type": "text", "text": format!("Error: {e}") }]
            }
        }),
    }
}

// --- Tool implementations ---

fn tool_list_templates() -> Result<String, DexError> {
    let templates = list_templates(&TemplateSource::Embedded)?;
    let lines: Vec<String> = templates
        .iter()
        .map(|t| format!("- **{}**: {}", t.name, t.description))
        .collect();
    Ok(format!("Available dex templates:\n\n{}", lines.join("\n")))
}

fn tool_get_template_variables(args: &Value) -> Result<String, DexError> {
    let name = args
        .get("template")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            DexError::Config(ConfigError::Invalid(
                "missing required argument: template".into(),
            ))
        })?;

    let template = load_template(&TemplateSource::Embedded, name)?;

    if template.variables.is_empty() {
        return Ok(format!("Template '{name}' has no variables."));
    }

    let mut lines = vec![format!("Variables for template '{name}':\n")];
    for v in &template.variables {
        let type_label = match v.var_type {
            VariableType::String => "string",
            VariableType::Bool => "bool",
            VariableType::Choice => "choice",
            VariableType::Multi => "multi",
        };
        let mut line = format!("- **{}** ({})", v.name, type_label);
        if v.required {
            line.push_str(" *required*");
        }
        line.push_str(&format!(": {}", v.prompt));
        if let Some(choices) = &v.choices {
            line.push_str(&format!(" [{}]", choices.join(", ")));
        }
        if let Some(default) = &v.default {
            line.push_str(&format!(" (default: {default})"));
        }
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

fn tool_scaffold_project(args: &Value) -> Result<String, DexError> {
    let template_name = args
        .get("template")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            DexError::Config(ConfigError::Invalid(
                "missing required argument: template".into(),
            ))
        })?;

    let dir = args.get("dir").and_then(|v| v.as_str()).ok_or_else(|| {
        DexError::Config(ConfigError::Invalid(
            "missing required argument: dir".into(),
        ))
    })?;

    let target = PathBuf::from(dir);
    if !target.exists() {
        std::fs::create_dir_all(&target).map_err(|source| DexError::Io {
            path: target.clone(),
            source,
        })?;
    }

    let template = load_template(&TemplateSource::Embedded, template_name)?;

    // Build variables: caller-supplied values take priority, then defaults.
    let provided = args
        .get("variables")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let default_project_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_project")
        .to_string();

    let mut variables: HashMap<String, minijinja::Value> = HashMap::new();

    for spec in &template.variables {
        let mv = if let Some(val) = provided.get(&spec.name) {
            json_value_to_minijinja(val)
        } else {
            default_for_spec(spec, &default_project_name)
        };
        variables.insert(spec.name.clone(), mv);
    }

    let result = scaffold(&template, &target, &variables)?;

    let file_list = result
        .files_created
        .iter()
        .map(|f| format!("  {}", f.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "Scaffolded {} files in '{dir}':\n{file_list}",
        result.files_created.len()
    ))
}

fn tool_scaffold_agent(args: &Value) -> Result<String, DexError> {
    let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        DexError::Config(ConfigError::Invalid(
            "missing required argument: name".into(),
        ))
    })?;

    let dir = args.get("dir").and_then(|v| v.as_str()).ok_or_else(|| {
        DexError::Config(ConfigError::Invalid(
            "missing required argument: dir".into(),
        ))
    })?;

    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let trigger = match args
        .get("trigger")
        .and_then(|v| v.as_str())
        .unwrap_or("user_request")
    {
        "schedule" => AgentTrigger::Schedule,
        "event" => AgentTrigger::Event,
        "upstream_system" => AgentTrigger::UpstreamSystem,
        _ => AgentTrigger::UserRequest,
    };

    let deploy_target = match args
        .get("deploy_target")
        .and_then(|v| v.as_str())
        .unwrap_or("job")
    {
        "serving_endpoint" => AgentDeployTarget::ServingEndpoint,
        "interactive" => AgentDeployTarget::Interactive,
        _ => AgentDeployTarget::Job,
    };

    let answers = AgentAnswers {
        name: name.to_string(),
        description,
        trigger,
        success_criteria: args
            .get("success_criteria")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        reads: args
            .get("reads")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        writes: args
            .get("writes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        handoff: args
            .get("handoff")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        autonomous: args
            .get("autonomous")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        example_input: args
            .get("example_input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        example_output: args
            .get("example_output")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        bad_output: args
            .get("bad_output")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        deploy_target,
    };

    let target = PathBuf::from(dir);
    if !target.exists() {
        std::fs::create_dir_all(&target).map_err(|source| DexError::Io {
            path: target.clone(),
            source,
        })?;
    }

    let result = scaffold_agent(&answers, &target)?;

    let file_list = result
        .files_created
        .iter()
        .map(|f| format!("  {}", f.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "Scaffolded agent '{}' ({} files) in '{}':\n{}\n\n---\n## System Prompt\n\n{}\n\n---\n## CLAUDE.md\n\n{}",
        name,
        result.files_created.len(),
        result.project_dir.display(),
        file_list,
        result.system_prompt,
        result.claude_md,
    ))
}

// --- Helpers ---

fn default_for_spec(
    spec: &dex_core::template::variables::VariableSpec,
    default_project_name: &str,
) -> minijinja::Value {
    match spec.var_type {
        VariableType::Bool => {
            let b = spec
                .default
                .as_ref()
                .and_then(|d| d.as_bool())
                .unwrap_or(false);
            minijinja::Value::from(b)
        }
        VariableType::Choice => {
            let v = spec
                .default
                .as_ref()
                .and_then(|d| d.as_str())
                .or_else(|| spec.choices.as_ref()?.first().map(String::as_str))
                .unwrap_or_default()
                .to_string();
            minijinja::Value::from(v)
        }
        _ => {
            if spec.name == "project_name" {
                minijinja::Value::from(default_project_name.to_string())
            } else {
                let v = spec
                    .default
                    .as_ref()
                    .and_then(|d| d.as_str())
                    .unwrap_or_default()
                    .to_string();
                minijinja::Value::from(v)
            }
        }
    }
}

fn json_value_to_minijinja(v: &Value) -> minijinja::Value {
    match v {
        Value::String(s) => minijinja::Value::from(s.clone()),
        Value::Bool(b) => minijinja::Value::from(*b),
        Value::Number(n) => n
            .as_i64()
            .map(minijinja::Value::from)
            .unwrap_or_else(|| minijinja::Value::from(n.to_string())),
        Value::Array(arr) => {
            let vals: Vec<minijinja::Value> = arr.iter().map(json_value_to_minijinja).collect();
            minijinja::Value::from(vals)
        }
        other => minijinja::Value::from(other.to_string()),
    }
}

fn error_response(id: &Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

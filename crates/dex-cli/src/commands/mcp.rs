//! `dex mcp serve` — JSON-RPC 2.0 MCP server over stdio.
//!
//! Exposes dex operations to MCP clients (Claude Desktop, Claude Code, etc.)
//! via the Model Context Protocol over stdin/stdout.
//!
//! Tools provided:
//! - `list_templates` — list all available dex templates
//! - `get_template_variables` — return variable specs for a named template
//! - `scaffold_project` — scaffold a project from a template into a directory
//!
//! Agent scaffolding uses `scaffold_project` with an agent template
//! (e.g. `agent-anthropic`, `agent-openai`, `agent-baml`).

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde_json::{Value, json};

use dex_core::context_map::write_context_map;
use dex_core::error::ConfigError;
use dex_core::skills::{InstallTarget, install_skills, load_pack};
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
                    "description": "Scaffold a new AI agent project. Batteries-included: generates AGENTS.md, MCP config, planner/reviewer stubs, eval suite, and registers the default + agent-dev skill packs so the project works out of the box in Claude Code, Cursor, Copilot, and any MCP client.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "sdk": {
                                "type": "string",
                                "enum": ["anthropic", "openai", "baml"],
                                "description": "SDK / framework for the agent. anthropic = Anthropic SDK (claude-*), openai = OpenAI SDK (gpt-*), baml = BAML typed functions (provider-agnostic)."
                            },
                            "dir": {
                                "type": "string",
                                "description": "Target directory path. Created if it does not exist."
                            },
                            "variables": {
                                "type": "object",
                                "description": "Variable values keyed by name (project_name, description, trigger, ai_tools, etc.). Call get_template_variables with template='agent-<sdk>' to see the full list. Missing variables use their defaults."
                            }
                        },
                        "required": ["sdk", "dir"]
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
        if let Some(pattern) = &v.validate {
            line.push_str(&format!(" (must match: {pattern})"));
        }
        if let Some(condition) = &v.when {
            line.push_str(&format!(" (only when: {condition})"));
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

    // Write .context-map.json (best-effort).
    let _ = write_context_map(&result, &template, &target, &variables);

    let file_list = result
        .files_created
        .iter()
        .map(|f| format!("  {}", f.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "Scaffolded {} files in '{dir}':\n{file_list}\n\nA .context-map.json was written to the project root.",
        result.files_created.len()
    ))
}

fn tool_scaffold_agent(args: &Value) -> Result<String, DexError> {
    let sdk = args.get("sdk").and_then(|v| v.as_str()).ok_or_else(|| {
        DexError::Config(ConfigError::Invalid(
            "missing required argument: sdk (anthropic | openai | baml)".into(),
        ))
    })?;

    let template_name = match sdk {
        "anthropic" => "agent-anthropic",
        "openai" => "agent-openai",
        "baml" => "agent-baml",
        other => {
            return Err(DexError::Config(ConfigError::Invalid(format!(
                "invalid sdk '{other}': expected one of anthropic, openai, baml"
            ))));
        }
    };

    let mut forwarded = args.clone();
    if let Some(obj) = forwarded.as_object_mut() {
        obj.insert(
            "template".to_string(),
            Value::String(template_name.to_string()),
        );
    }

    let body = tool_scaffold_project(&forwarded)?;

    // Install agent skill packs to all four assistant targets so the project is
    // ready out of the box for Claude Code, Cursor, Copilot, and generic MCP clients.
    let dir = args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");
    let target_dir = PathBuf::from(dir);
    let targets = [
        InstallTarget::Claude,
        InstallTarget::Cursor,
        InstallTarget::Copilot,
        InstallTarget::Generic,
    ];
    let mut skills_written = 0usize;
    let mut skills_notes: Vec<String> = Vec::new();
    for pack_name in ["default", "agent-dev"] {
        match load_pack(pack_name, None, &[]) {
            Ok(pack) => match install_skills(&pack, &target_dir, &targets) {
                Ok(res) => skills_written += res.files_written.len(),
                Err(e) => skills_notes.push(format!("  - {pack_name}: install failed ({e})")),
            },
            Err(e) => skills_notes.push(format!("  - {pack_name}: not loadable ({e})")),
        }
    }

    let skills_summary = if skills_notes.is_empty() {
        format!(
            "\n\nInstalled {skills_written} skill files for claude/cursor/copilot/generic targets."
        )
    } else {
        format!(
            "\n\nInstalled {skills_written} skill files. Notes:\n{}",
            skills_notes.join("\n")
        )
    };

    Ok(format!(
        "Scaffolded agent ({sdk}) — skills and MCP config included.\n\n{body}{skills_summary}"
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

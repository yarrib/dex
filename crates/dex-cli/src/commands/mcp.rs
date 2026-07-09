//! `dex mcp serve` — JSON-RPC 2.0 MCP server over stdio.
//!
//! Exposes dex operations to MCP clients (Claude Desktop, Claude Code, Cursor,
//! or any MCP-capable tool) via the Model Context Protocol over stdin/stdout.
//!
//! Tools provided:
//! - `list_templates` — list all available dex templates
//! - `get_template_variables` — return variable specs for a named template
//! - `scaffold_project` — scaffold a project from a template into a directory
//! - `scaffold_agent` — scaffold an AI agent (sdk = anthropic|openai|baml);
//!   thin wrapper over `scaffold_project` that also installs skill packs into
//!   the targets named in the `ai_tools` variable (default: all four).

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;
use dialoguer::MultiSelect;
use serde_json::{Value, json};

use dex_core::context_map::write_context_map;
use dex_core::error::ConfigError;
use dex_core::mcp::{McpClient, apply_mcp_plan, plan_mcp_client};
use dex_core::skills::{InstallTarget, install_skills, load_pack};
use dex_core::template::TemplateSource;
use dex_core::template::registry::{list_templates, load_template};
use dex_core::template::variables::VariableType;
use dex_core::update::{SourceKind, record_project_state};
use dex_core::{DexError, scaffold};

use crate::output;

#[derive(Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub cmd: McpCommand,
}

#[derive(Subcommand)]
pub enum McpCommand {
    /// Start the MCP server on stdio (JSON-RPC 2.0).
    Serve,
    /// Wire the dex MCP server into AI coding assistants' config files.
    Install(InstallArgs),
}

#[derive(Args)]
pub struct InstallArgs {
    /// Client to wire up (repeatable). One of: claude-code, claude-desktop,
    /// cursor, vscode, codex, zed, antigravity. Omit for an interactive picker.
    #[arg(long = "client", value_name = "NAME")]
    clients: Vec<String>,

    /// Wire up every supported client.
    #[arg(long)]
    all: bool,

    /// Project directory for project-scoped clients (claude-code, vscode).
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Executable the client should launch. Defaults to "dex"; pass an absolute
    /// path if dex is not on the client's PATH (common for GUI apps).
    #[arg(long)]
    command: Option<String>,

    /// Show what would be written without modifying any files.
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: McpArgs) -> Result<(), DexError> {
    match args.cmd {
        McpCommand::Serve => serve(),
        McpCommand::Install(a) => install(a),
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

// --- mcp install ---

fn install(args: InstallArgs) -> Result<(), DexError> {
    let command = args.command.as_deref().unwrap_or("dex").to_string();

    let project_dir = PathBuf::from(&args.dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.dir));

    // Resolve the set of clients to wire up.
    let clients = select_clients(&args)?;
    if clients.is_empty() {
        output::print_dim("No clients selected.");
        return Ok(());
    }

    println!(
        "\n{} (command: {})\n",
        style("dex mcp install").bold(),
        style(&command).cyan()
    );

    let mut applied = 0usize;
    for client in clients {
        match plan_mcp_client(client, &project_dir, &command) {
            Ok(plan) => {
                if args.dry_run {
                    let verb = if plan.created { "create" } else { "update" };
                    println!(
                        "  {} {} — would {} {}",
                        style("•").cyan(),
                        client.display_name(),
                        verb,
                        style(plan.path.display()).dim()
                    );
                    println!("{}\n", indent(&plan.content));
                } else {
                    apply_mcp_plan(&plan)?;
                    let verb = if plan.created { "created" } else { "updated" };
                    println!(
                        "  {} {} — {} {}",
                        style("✓").green(),
                        client.display_name(),
                        verb,
                        style(plan.path.display()).dim()
                    );
                    applied += 1;
                }
            }
            // Don't abort the whole run if one client can't be resolved
            // (e.g. no home directory); report and continue.
            Err(e) => {
                output::print_warning(&format!("{}: {e}", client.display_name()));
            }
        }
    }

    if args.dry_run {
        println!(
            "{}",
            style("Dry run — no files were modified. Re-run without --dry-run to apply.").dim()
        );
    } else {
        println!(
            "\n{} wired up {} client{}.",
            style("Done!").green().bold(),
            applied,
            if applied == 1 { "" } else { "s" }
        );
        if command == "dex" {
            println!(
                "{}",
                style(
                    "Tip: GUI clients may not see `dex` on their PATH. If a server fails to \
                     start, re-run with --command \"$(command -v dex)\"."
                )
                .dim()
            );
        }
        println!(
            "{}",
            style("Restart the client (or reload its MCP servers) to pick up the change.").dim()
        );
    }

    Ok(())
}

/// Determine which clients to target: `--all`, explicit `--client` flags, or an
/// interactive multi-select.
fn select_clients(args: &InstallArgs) -> Result<Vec<McpClient>, DexError> {
    if args.all {
        return Ok(McpClient::ALL.to_vec());
    }

    if !args.clients.is_empty() {
        return args
            .clients
            .iter()
            .map(|c| McpClient::parse(c))
            .collect::<Result<Vec<_>, _>>();
    }

    // Interactive picker over all clients.
    let labels: Vec<&str> = McpClient::ALL.iter().map(|c| c.display_name()).collect();
    let selections = MultiSelect::new()
        .with_prompt("Select clients to wire dex into (space to toggle, enter to confirm)")
        .items(&labels)
        .interact()
        .map_err(io_error)?;

    Ok(selections.into_iter().map(|i| McpClient::ALL[i]).collect())
}

fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("      {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
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

    // Record `.dex/` state so the scaffolded project is updatable via
    // `dex update` (best-effort; MCP templates are always embedded, so the
    // recorded ref is the template version).
    let _ = record_project_state(
        &target,
        &template,
        SourceKind::Embedded,
        None,
        template.meta.version.clone(),
        env!("CARGO_PKG_VERSION"),
        &variables,
    );

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

    // Install agent skill packs. Honor the caller's `ai_tools` variable so the
    // MCP path matches `dex agent new`'s behavior exactly; default to all four
    // targets when unspecified.
    let dir = args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");
    let target_dir = PathBuf::from(dir);
    let targets: Vec<InstallTarget> = args
        .get("variables")
        .and_then(|v| v.get("ai_tools"))
        .and_then(|v| v.as_str())
        .map(|s| {
            s.split(',')
                .filter_map(|tok| InstallTarget::parse(tok.trim()).ok())
                .collect::<Vec<_>>()
        })
        .filter(|v: &Vec<InstallTarget>| !v.is_empty())
        .unwrap_or_else(|| {
            vec![
                InstallTarget::Claude,
                InstallTarget::Cursor,
                InstallTarget::Copilot,
                InstallTarget::Generic,
            ]
        });
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

    let target_names = targets
        .iter()
        .map(|t| t.as_str())
        .collect::<Vec<_>>()
        .join("/");
    let skills_summary = if skills_notes.is_empty() {
        format!("\n\nInstalled {skills_written} skill files for {target_names} targets.")
    } else {
        format!(
            "\n\nInstalled {skills_written} skill files for {target_names} targets. Notes:\n{}",
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

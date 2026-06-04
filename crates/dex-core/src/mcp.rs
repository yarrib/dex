//! Wire the dex MCP server into AI coding assistants' config files.
//!
//! All logic here is pure business logic: given a client and the relevant base
//! directories, compute the config file path and merge a `dex` server entry into
//! the existing config **without disturbing other servers**. No terminal output —
//! the CLI layer renders the results.
//!
//! Every client launches the same command (`dex mcp serve`) over stdio; clients
//! differ only in (a) where the config file lives and (b) its surrounding shape:
//!
//! - `mcpServers` JSON — Claude Code, Claude Desktop, Cursor, Antigravity
//! - `servers` JSON (with `"type": "stdio"`) — VS Code / GitHub Copilot
//! - `context_servers` JSON — Zed
//! - `mcp_servers` TOML — OpenAI Codex CLI

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::error::{DexError, McpError};

/// The MCP server name written into each client config.
const SERVER_KEY: &str = "dex";

/// A supported MCP client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpClient {
    /// Claude Code — `.mcp.json` at the project root (`mcpServers`).
    ClaudeCode,
    /// Claude Desktop — OS-specific user config (`mcpServers`).
    ClaudeDesktop,
    /// Cursor — `~/.cursor/mcp.json` (`mcpServers`).
    Cursor,
    /// VS Code / GitHub Copilot — `.vscode/mcp.json` (`servers`, `type: stdio`).
    VsCode,
    /// OpenAI Codex CLI — `~/.codex/config.toml` (`[mcp_servers.dex]`).
    Codex,
    /// Zed — `~/.config/zed/settings.json` (`context_servers`).
    Zed,
    /// Google Antigravity — `~/.gemini/config/mcp_config.json` (`mcpServers`).
    Antigravity,
}

/// The shape of a client's config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    /// JSON with a top-level `mcpServers` object.
    McpServers,
    /// JSON with a top-level `servers` object; entries carry `"type": "stdio"`.
    Servers,
    /// JSON with a top-level `context_servers` object (Zed).
    ContextServers,
    /// TOML with `[mcp_servers.<name>]` tables (Codex).
    CodexToml,
}

impl McpClient {
    /// All supported clients.
    pub const ALL: [McpClient; 7] = [
        McpClient::ClaudeCode,
        McpClient::ClaudeDesktop,
        McpClient::Cursor,
        McpClient::VsCode,
        McpClient::Codex,
        McpClient::Zed,
        McpClient::Antigravity,
    ];

    /// Parse a client from a string identifier (case-insensitive, with aliases).
    pub fn parse(s: &str) -> Result<Self, DexError> {
        match s.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "claude-code" | "claudecode" | "claude" => Ok(McpClient::ClaudeCode),
            "claude-desktop" | "claudedesktop" | "desktop" => Ok(McpClient::ClaudeDesktop),
            "cursor" => Ok(McpClient::Cursor),
            "vscode" | "vs-code" | "copilot" | "github-copilot" => Ok(McpClient::VsCode),
            "codex" => Ok(McpClient::Codex),
            "zed" => Ok(McpClient::Zed),
            "antigravity" => Ok(McpClient::Antigravity),
            other => Err(DexError::Mcp(McpError::UnknownClient(other.to_string()))),
        }
    }

    /// The canonical string identifier used on the CLI.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            McpClient::ClaudeCode => "claude-code",
            McpClient::ClaudeDesktop => "claude-desktop",
            McpClient::Cursor => "cursor",
            McpClient::VsCode => "vscode",
            McpClient::Codex => "codex",
            McpClient::Zed => "zed",
            McpClient::Antigravity => "antigravity",
        }
    }

    /// Human-readable name including where the config lives.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            McpClient::ClaudeCode => "Claude Code (.mcp.json in project)",
            McpClient::ClaudeDesktop => "Claude Desktop (user config)",
            McpClient::Cursor => "Cursor (~/.cursor/mcp.json)",
            McpClient::VsCode => "VS Code / GitHub Copilot (.vscode/mcp.json in project)",
            McpClient::Codex => "OpenAI Codex CLI (~/.codex/config.toml)",
            McpClient::Zed => "Zed (~/.config/zed/settings.json)",
            McpClient::Antigravity => "Google Antigravity (~/.gemini/config/mcp_config.json)",
        }
    }

    /// Whether the config lives in the project directory (vs. user-global).
    #[must_use]
    pub fn is_project_scoped(&self) -> bool {
        matches!(self, McpClient::ClaudeCode | McpClient::VsCode)
    }

    fn format(&self) -> Format {
        match self {
            McpClient::ClaudeCode | McpClient::ClaudeDesktop | McpClient::Antigravity => {
                Format::McpServers
            }
            McpClient::Cursor => Format::McpServers,
            McpClient::VsCode => Format::Servers,
            McpClient::Zed => Format::ContextServers,
            McpClient::Codex => Format::CodexToml,
        }
    }
}

/// A planned change to a single client's config file.
#[derive(Debug)]
pub struct McpInstallPlan {
    pub client: McpClient,
    /// The config file that will be written.
    pub path: PathBuf,
    /// `true` if the file does not exist yet (will be created).
    pub created: bool,
    /// The full new file contents.
    pub content: String,
}

/// Compute the change needed to wire dex into `client`, without writing anything.
///
/// `project_dir` is used for project-scoped clients (Claude Code, VS Code).
/// `command` is the executable the client should launch (usually `"dex"`, or an
/// absolute path when `dex` is not on the client's `PATH`).
pub fn plan_mcp_client(
    client: McpClient,
    project_dir: &Path,
    command: &str,
) -> Result<McpInstallPlan, DexError> {
    let home = dirs::home_dir();
    let path = resolve_config_path(client, project_dir, home.as_deref())?;

    let existing = if path.exists() {
        std::fs::read_to_string(&path).map_err(|source| DexError::Io {
            path: path.clone(),
            source,
        })?
    } else {
        String::new()
    };

    let content = build_client_config(client, &existing, command, &path.display().to_string())?;

    Ok(McpInstallPlan {
        client,
        created: !path.exists(),
        path,
        content,
    })
}

/// Write a previously computed plan to disk, creating parent directories.
pub fn apply_mcp_plan(plan: &McpInstallPlan) -> Result<(), DexError> {
    if let Some(parent) = plan.path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(&plan.path, &plan.content).map_err(|source| DexError::Io {
        path: plan.path.clone(),
        source,
    })
}

// ---------------------------------------------------------------------------
// Path resolution (pure)
// ---------------------------------------------------------------------------

/// Resolve the config file path for `client`. `home` is required for
/// user-scoped clients; project-scoped clients use `project_dir`.
pub(crate) fn resolve_config_path(
    client: McpClient,
    project_dir: &Path,
    home: Option<&Path>,
) -> Result<PathBuf, DexError> {
    let path = match client {
        McpClient::ClaudeCode => project_dir.join(".mcp.json"),
        McpClient::VsCode => project_dir.join(".vscode").join("mcp.json"),
        McpClient::Cursor => home_path(home, client)?.join(".cursor").join("mcp.json"),
        McpClient::Codex => home_path(home, client)?.join(".codex").join("config.toml"),
        McpClient::Zed => home_path(home, client)?
            .join(".config")
            .join("zed")
            .join("settings.json"),
        McpClient::Antigravity => home_path(home, client)?
            .join(".gemini")
            .join("config")
            .join("mcp_config.json"),
        McpClient::ClaudeDesktop => claude_desktop_path(home_path(home, client)?),
    };
    Ok(path)
}

fn home_path(home: Option<&Path>, client: McpClient) -> Result<&Path, DexError> {
    home.ok_or(DexError::Mcp(McpError::HomeDirNotFound(client.as_str())))
}

/// Claude Desktop's config path is OS-specific.
fn claude_desktop_path(home: &Path) -> PathBuf {
    match std::env::consts::OS {
        "macos" => home
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json"),
        "windows" => home
            .join("AppData")
            .join("Roaming")
            .join("Claude")
            .join("claude_desktop_config.json"),
        // Linux and others.
        _ => home
            .join(".config")
            .join("Claude")
            .join("claude_desktop_config.json"),
    }
}

// ---------------------------------------------------------------------------
// Config building / merging (pure)
// ---------------------------------------------------------------------------

/// Build the new file contents for `client` by merging a `dex` server entry into
/// `existing` (which may be empty). `path` is used only for error messages.
pub fn build_client_config(
    client: McpClient,
    existing: &str,
    command: &str,
    path: &str,
) -> Result<String, DexError> {
    match client.format() {
        Format::McpServers => merge_json(existing, "mcpServers", json_entry(command), path),
        Format::Servers => merge_json(existing, "servers", servers_entry(command), path),
        Format::ContextServers => {
            merge_json(existing, "context_servers", context_entry(command), path)
        }
        Format::CodexToml => merge_codex_toml(existing, command, path),
    }
}

fn json_entry(command: &str) -> Value {
    json!({ "command": command, "args": ["mcp", "serve"] })
}

fn servers_entry(command: &str) -> Value {
    json!({ "type": "stdio", "command": command, "args": ["mcp", "serve"] })
}

fn context_entry(command: &str) -> Value {
    json!({ "source": "custom", "command": command, "args": ["mcp", "serve"], "env": {} })
}

fn merge_json(existing: &str, top_key: &str, entry: Value, path: &str) -> Result<String, DexError> {
    let mut root: Value = if existing.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(existing).map_err(|e| McpError::Parse {
            path: path.to_string(),
            message: e.to_string(),
        })?
    };

    let obj = root.as_object_mut().ok_or_else(|| McpError::NotAnObject {
        path: path.to_string(),
        key: "<root>".to_string(),
    })?;

    let servers = obj.entry(top_key).or_insert_with(|| json!({}));
    let servers_obj = servers
        .as_object_mut()
        .ok_or_else(|| McpError::NotAnObject {
            path: path.to_string(),
            key: top_key.to_string(),
        })?;
    servers_obj.insert(SERVER_KEY.to_string(), entry);

    let mut out = serde_json::to_string_pretty(&root).map_err(|e| McpError::Serialize {
        path: path.to_string(),
        message: e.to_string(),
    })?;
    out.push('\n');
    Ok(out)
}

fn merge_codex_toml(existing: &str, command: &str, path: &str) -> Result<String, DexError> {
    let mut root: toml::Value = if existing.trim().is_empty() {
        toml::Value::Table(toml::value::Table::new())
    } else {
        existing
            .parse::<toml::Value>()
            .map_err(|e| McpError::Parse {
                path: path.to_string(),
                message: e.to_string(),
            })?
    };

    let table = root.as_table_mut().ok_or_else(|| McpError::NotAnObject {
        path: path.to_string(),
        key: "<root>".to_string(),
    })?;

    let servers = table
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let servers_tbl = servers
        .as_table_mut()
        .ok_or_else(|| McpError::NotAnObject {
            path: path.to_string(),
            key: "mcp_servers".to_string(),
        })?;

    let mut dex = toml::value::Table::new();
    dex.insert(
        "command".to_string(),
        toml::Value::String(command.to_string()),
    );
    dex.insert(
        "args".to_string(),
        toml::Value::Array(vec![
            toml::Value::String("mcp".to_string()),
            toml::Value::String("serve".to_string()),
        ]),
    );
    servers_tbl.insert(SERVER_KEY.to_string(), toml::Value::Table(dex));

    toml::to_string_pretty(&root).map_err(|e| {
        DexError::Mcp(McpError::Serialize {
            path: path.to_string(),
            message: e.to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_aliases() {
        assert_eq!(McpClient::parse("vscode").unwrap(), McpClient::VsCode);
        assert_eq!(McpClient::parse("copilot").unwrap(), McpClient::VsCode);
        assert_eq!(
            McpClient::parse("Claude_Code").unwrap(),
            McpClient::ClaudeCode
        );
        assert!(McpClient::parse("nope").is_err());
    }

    #[test]
    fn project_scoped_paths_use_project_dir() {
        let proj = Path::new("/tmp/proj");
        let cc = resolve_config_path(McpClient::ClaudeCode, proj, None).unwrap();
        assert_eq!(cc, Path::new("/tmp/proj/.mcp.json"));
        let vs = resolve_config_path(McpClient::VsCode, proj, None).unwrap();
        assert_eq!(vs, Path::new("/tmp/proj/.vscode/mcp.json"));
    }

    #[test]
    fn user_scoped_paths_need_home() {
        let proj = Path::new("/tmp/proj");
        assert!(resolve_config_path(McpClient::Cursor, proj, None).is_err());

        let home = Path::new("/home/u");
        let cursor = resolve_config_path(McpClient::Cursor, proj, Some(home)).unwrap();
        assert_eq!(cursor, Path::new("/home/u/.cursor/mcp.json"));
        let codex = resolve_config_path(McpClient::Codex, proj, Some(home)).unwrap();
        assert_eq!(codex, Path::new("/home/u/.codex/config.toml"));
        let zed = resolve_config_path(McpClient::Zed, proj, Some(home)).unwrap();
        assert_eq!(zed, Path::new("/home/u/.config/zed/settings.json"));
        let ag = resolve_config_path(McpClient::Antigravity, proj, Some(home)).unwrap();
        assert_eq!(ag, Path::new("/home/u/.gemini/config/mcp_config.json"));
    }

    #[test]
    fn merge_into_empty_mcpservers() {
        let out = build_client_config(McpClient::ClaudeCode, "", "dex", "/tmp/.mcp.json").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["mcpServers"]["dex"]["command"], "dex");
        assert_eq!(v["mcpServers"]["dex"]["args"][0], "mcp");
        assert_eq!(v["mcpServers"]["dex"]["args"][1], "serve");
    }

    #[test]
    fn merge_preserves_existing_servers() {
        let existing = r#"{
  "mcpServers": {
    "other": { "command": "foo", "args": ["bar"] }
  }
}"#;
        let out = build_client_config(McpClient::Cursor, existing, "dex", "/tmp/mcp.json").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        // Existing server untouched.
        assert_eq!(v["mcpServers"]["other"]["command"], "foo");
        // dex added.
        assert_eq!(v["mcpServers"]["dex"]["command"], "dex");
    }

    #[test]
    fn vscode_uses_servers_key_and_stdio_type() {
        let out =
            build_client_config(McpClient::VsCode, "", "dex", "/tmp/.vscode/mcp.json").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["servers"]["dex"]["type"], "stdio");
        assert_eq!(v["servers"]["dex"]["command"], "dex");
        assert!(v.get("mcpServers").is_none());
    }

    #[test]
    fn zed_uses_context_servers_with_custom_source() {
        let out = build_client_config(McpClient::Zed, "", "dex", "/tmp/settings.json").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["context_servers"]["dex"]["source"], "custom");
        assert_eq!(v["context_servers"]["dex"]["command"], "dex");
        assert!(v["context_servers"]["dex"]["env"].is_object());
    }

    #[test]
    fn codex_merges_toml_and_preserves_other_tables() {
        let existing =
            "[model]\nname = \"gpt-5\"\n\n[mcp_servers.other]\ncommand = \"foo\"\nargs = [\"x\"]\n";
        let out = build_client_config(
            McpClient::Codex,
            existing,
            "/abs/dex",
            "/home/u/.codex/config.toml",
        )
        .unwrap();
        let v: toml::Value = out.parse().unwrap();
        assert_eq!(v["model"]["name"].as_str(), Some("gpt-5"));
        assert_eq!(v["mcp_servers"]["other"]["command"].as_str(), Some("foo"));
        assert_eq!(
            v["mcp_servers"]["dex"]["command"].as_str(),
            Some("/abs/dex")
        );
        assert_eq!(v["mcp_servers"]["dex"]["args"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn merge_rejects_non_object_root() {
        let err = build_client_config(McpClient::Cursor, "[1,2,3]", "dex", "/tmp/x.json");
        assert!(err.is_err());
    }

    #[test]
    fn custom_command_path_is_used() {
        let out = build_client_config(
            McpClient::ClaudeDesktop,
            "",
            "/Users/me/.local/bin/dex",
            "/tmp/cfg.json",
        )
        .unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["mcpServers"]["dex"]["command"],
            "/Users/me/.local/bin/dex"
        );
    }
}

//! Integration tests for the `dex` CLI binary.
//!
//! These tests invoke the compiled binary via `assert_cmd` to verify that
//! commands parse correctly, produce the expected output, and exit with the
//! right status code.

use assert_cmd::Command;

fn dex() -> Command {
    Command::cargo_bin("dex").expect("dex binary not found — run `cargo build` first")
}

/// Send one or more newline-separated JSON-RPC lines to `dex mcp serve` via
/// stdin and return the collected stdout lines as a Vec<serde_json::Value>.
fn mcp_call(requests: &str) -> Vec<serde_json::Value> {
    let output = dex()
        .args(["mcp", "serve"])
        .write_stdin(requests.to_string())
        .output()
        .expect("failed to run dex mcp serve");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("MCP response is not valid JSON"))
        .collect()
}

#[test]
fn version_flag_exits_success() {
    dex().arg("--version").assert().success();
}

#[test]
fn help_flag_exits_success() {
    dex().arg("--help").assert().success();
}

#[test]
fn init_help_exits_success() {
    dex().args(["init", "--help"]).assert().success();
}

#[test]
fn init_unknown_template_exits_failure() {
    let dir = tempfile::tempdir().unwrap();
    dex()
        .args([
            "init",
            "--template",
            "does-not-exist",
            "--no-prompt",
            "--dir",
        ])
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn init_with_preset_skips_prompts_and_applies_values() {
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("presetproject");
    std::fs::create_dir(&project_dir).unwrap();

    // Write a temporary presets file with a known profile.
    let presets_file = base.path().join("presets.toml");
    std::fs::write(
        &presets_file,
        "[profiles.myprof]\npython_version = \"3.11\"\n",
    )
    .unwrap();

    dex()
        .args([
            "init",
            "--template",
            "default",
            "--no-prompt",
            "--preset",
            "myprof",
            "--presets-file",
        ])
        .arg(&presets_file)
        .arg("--dir")
        .arg(&project_dir)
        .assert()
        .success();

    // Scaffolded files should exist.
    let entries: Vec<_> = std::fs::read_dir(&project_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "expected scaffolded files");
}

#[test]
fn init_with_unknown_preset_exits_failure() {
    let dir = tempfile::tempdir().unwrap();
    let presets_file = dir.path().join("presets.toml");
    std::fs::write(&presets_file, "[profiles.other]\nfoo = \"bar\"\n").unwrap();

    dex()
        .args([
            "init",
            "--template",
            "default",
            "--no-prompt",
            "--preset",
            "does-not-exist",
            "--presets-file",
        ])
        .arg(&presets_file)
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn init_default_template_no_prompt_creates_files() {
    // Use a directory name that satisfies the project_name regex: ^[a-z][a-z0-9_-]*$
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("myproject");
    std::fs::create_dir(&project_dir).unwrap();

    dex()
        .args(["init", "--template", "default", "--no-prompt", "--dir"])
        .arg(&project_dir)
        .assert()
        .success();

    // The default template should produce at least one file.
    let entries: Vec<_> = std::fs::read_dir(&project_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !entries.is_empty(),
        "expected scaffolded files in {project_dir:?}"
    );
}

// --- MCP server integration tests ---

#[test]
fn mcp_serve_help_exits_success() {
    dex().args(["mcp", "--help"]).assert().success();
}

#[test]
fn mcp_list_templates_returns_known_templates() {
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_templates","arguments":{}}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("expected text content");

    assert!(
        text.contains("default"),
        "missing 'default' template in list"
    );
    assert!(
        text.contains("dabs-package"),
        "missing 'dabs-package' template in list"
    );
    assert!(
        text.contains("python-package"),
        "missing 'python-package' template in list"
    );
}

#[test]
fn mcp_get_template_variables_returns_specs() {
    let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_template_variables","arguments":{"template":"dabs-package"}}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("expected text content");

    assert!(
        text.contains("project_name"),
        "missing project_name variable"
    );
    assert!(
        text.contains("python_version"),
        "missing python_version variable"
    );
    assert!(
        text.contains("include_notebook"),
        "missing include_notebook variable"
    );
}

#[test]
fn mcp_get_template_variables_exposes_validate_pattern() {
    // agent-anthropic has a validate regex on project_name.
    let req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_template_variables","arguments":{"template":"agent-anthropic"}}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("expected text content");

    assert!(
        text.contains("must match:"),
        "expected 'must match:' annotation for validated variables in agent-anthropic:\n{text}"
    );
}

#[test]
fn mcp_get_template_variables_unknown_template_returns_error() {
    let req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_template_variables","arguments":{"template":"does-not-exist"}}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    // MCP returns errors as isError:true in the result (not a JSON-RPC error).
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(is_error, "expected isError:true for unknown template");
}

#[test]
fn mcp_scaffold_project_creates_files() {
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("mcp-test-project");

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{{"name":"scaffold_project","arguments":{{"template":"default","dir":"{}","variables":{{"project_name":"mcptestproject"}}}}}}}}"#,
        project_dir.display()
    );
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(
        !is_error,
        "scaffold_project returned an error: {responses:?}"
    );

    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("expected text content");
    assert!(
        text.contains("Scaffolded"),
        "expected 'Scaffolded' in response: {text}"
    );

    // Directory and at least one file should exist.
    assert!(project_dir.exists(), "project directory was not created");
    let entries: Vec<_> = std::fs::read_dir(&project_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !entries.is_empty(),
        "expected scaffolded files in {project_dir:?}"
    );
}

#[test]
fn mcp_scaffold_project_missing_template_returns_error() {
    let base = tempfile::tempdir().unwrap();
    let req = format!(
        r#"{{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{{"name":"scaffold_project","arguments":{{"template":"no-such-template","dir":"{}"}}}}}}"#,
        base.path().display()
    );
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(is_error, "expected isError:true for unknown template");
}

#[test]
fn mcp_initialize_returns_server_info() {
    let req = r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0]["result"]["serverInfo"]["name"].as_str(),
        Some("dex")
    );
    assert_eq!(
        responses[0]["result"]["protocolVersion"].as_str(),
        Some("2024-11-05")
    );
}

#[test]
fn mcp_tools_list_returns_all_tools() {
    let req = r#"{"jsonrpc":"2.0","id":8,"method":"tools/list","params":{}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"]
        .as_array()
        .expect("expected tools array");

    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    for expected in [
        "list_templates",
        "get_template_variables",
        "scaffold_project",
        "scaffold_agent",
    ] {
        assert!(names.contains(&expected), "missing tool: {expected}");
    }
}

#[test]
fn mcp_scaffold_agent_creates_files_and_installs_skills() {
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("mcpagent");

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{{"name":"scaffold_agent","arguments":{{"sdk":"anthropic","dir":"{}","variables":{{"project_name":"mcpagent","description":"test agent"}}}}}}}}"#,
        project_dir.display()
    );
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(!is_error, "scaffold_agent returned error: {responses:?}");

    // AGENTS.md and planner/reviewer stubs should exist.
    let agents_md = project_dir.join("AGENTS.md");
    assert!(agents_md.exists(), "missing AGENTS.md");
    assert!(project_dir.join(".mcp.json").exists(), "missing .mcp.json");
    assert!(
        project_dir.join("src/mcpagent/tools/planner.py").exists(),
        "missing planner.py"
    );
    assert!(
        project_dir.join("src/mcpagent/tools/reviewer.py").exists(),
        "missing reviewer.py"
    );

    // AGENTS.md must actually render — no raw Jinja tokens, project_name resolved.
    let agents_body = std::fs::read_to_string(&agents_md).unwrap();
    assert!(
        agents_body.contains("# AGENTS.md — mcpagent"),
        "AGENTS.md header not rendered: {agents_body}"
    );
    assert!(
        !agents_body.contains("{{"),
        "AGENTS.md contains unrendered Jinja tokens"
    );

    // Skills installed for all four targets when ai_tools is unset.
    assert!(
        project_dir.join(".claude/commands").is_dir(),
        "expected .claude/commands to be populated"
    );
    assert!(
        project_dir.join(".cursor/rules").is_dir(),
        "expected .cursor/rules to be populated"
    );
    assert!(
        project_dir.join(".github/copilot-instructions.md").exists(),
        "expected .github/copilot-instructions.md to be written"
    );
    assert!(
        project_dir.join(".ai-skills/commands").is_dir(),
        "expected .ai-skills/commands to be populated"
    );
}

#[test]
fn mcp_scaffold_agent_honors_ai_tools_variable() {
    // When ai_tools is narrowed, only named targets get skill packs.
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("cursoronly");

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{{"name":"scaffold_agent","arguments":{{"sdk":"anthropic","dir":"{}","variables":{{"project_name":"cursoronly","ai_tools":"cursor"}}}}}}}}"#,
        project_dir.display()
    );
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(!is_error, "scaffold_agent returned error: {responses:?}");

    assert!(
        project_dir.join(".cursor/rules").is_dir(),
        "cursor target should have been installed"
    );
    assert!(
        !project_dir.join(".claude").exists(),
        "claude target should NOT have been installed when ai_tools=cursor"
    );
    assert!(
        !project_dir.join(".github/copilot-instructions.md").exists(),
        "copilot target should NOT have been installed when ai_tools=cursor"
    );
    assert!(
        !project_dir.join(".ai-skills").exists(),
        "generic target should NOT have been installed when ai_tools=cursor"
    );
}

#[test]
fn init_slugifies_hyphenated_dir_name_to_valid_python_module() {
    // `dex init --dir foo-bar` must produce `src/foo_bar/` — not `src/foo-bar/`
    // which would be an invalid Python import path.
    let base = tempfile::tempdir().unwrap();
    let project_dir = base.path().join("my-hyphenated-agent");
    std::fs::create_dir(&project_dir).unwrap();

    dex()
        .args([
            "init",
            "--template",
            "agent-anthropic",
            "--no-prompt",
            "--dir",
        ])
        .arg(&project_dir)
        .assert()
        .success();

    assert!(
        project_dir.join("src/my_hyphenated_agent").is_dir(),
        "expected slugified module path; got entries: {:?}",
        std::fs::read_dir(project_dir.join("src"))
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect::<Vec<_>>()
    );
    assert!(
        !project_dir.join("src/my-hyphenated-agent").exists(),
        "hyphenated path must not have been created"
    );
}

#[test]
fn skills_init_yes_without_packs_and_targets_exits_failure() {
    // --yes requires --packs and --targets so it never deadlocks on a prompt.
    let dir = tempfile::tempdir().unwrap();
    dex()
        .args(["skills", "init", "--yes", "--dir"])
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn mcp_scaffold_agent_invalid_sdk_returns_error() {
    let base = tempfile::tempdir().unwrap();
    let req = format!(
        r#"{{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{{"name":"scaffold_agent","arguments":{{"sdk":"haskell","dir":"{}"}}}}}}"#,
        base.path().display()
    );
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let is_error = responses[0]["result"]["isError"].as_bool().unwrap_or(false);
    assert!(is_error, "expected error for invalid sdk");
}

// --- mcp install integration tests ---

#[test]
fn mcp_install_help_exits_success() {
    dex().args(["mcp", "install", "--help"]).assert().success();
}

#[test]
fn mcp_install_unknown_client_exits_failure() {
    dex()
        .args(["mcp", "install", "--client", "notaneditor"])
        .assert()
        .failure();
}

#[test]
fn mcp_install_writes_project_scoped_configs() {
    let base = tempfile::tempdir().unwrap();

    dex()
        .args([
            "mcp",
            "install",
            "--client",
            "claude-code",
            "--client",
            "vscode",
            "--dir",
        ])
        .arg(base.path())
        .assert()
        .success();

    // Claude Code: .mcp.json with mcpServers.dex
    let mcp_json = base.path().join(".mcp.json");
    assert!(mcp_json.exists(), "expected {mcp_json:?} to be created");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
    assert_eq!(v["mcpServers"]["dex"]["command"], "dex");

    // VS Code: .vscode/mcp.json with servers.dex (type stdio)
    let vscode_json = base.path().join(".vscode").join("mcp.json");
    assert!(
        vscode_json.exists(),
        "expected {vscode_json:?} to be created"
    );
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&vscode_json).unwrap()).unwrap();
    assert_eq!(v["servers"]["dex"]["type"], "stdio");
}

#[test]
fn mcp_install_dry_run_writes_nothing() {
    let base = tempfile::tempdir().unwrap();

    dex()
        .args([
            "mcp",
            "install",
            "--client",
            "claude-code",
            "--dry-run",
            "--dir",
        ])
        .arg(base.path())
        .assert()
        .success();

    assert!(
        !base.path().join(".mcp.json").exists(),
        "dry-run must not write any files"
    );
}

// --- dex context sync -------------------------------------------------------

/// Run a git command in `dir`, panicking on failure. Uses inline identity and
/// disables commit signing so the test doesn't depend on global git config.
fn git(dir: &std::path::Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(dir)
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(args)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {args:?} failed");
}

fn commit_file(dir: &std::path::Path, path: &str, contents: &str, message: &str) {
    let full = dir.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, contents).unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", message]);
}

#[test]
fn context_sync_builds_graph_in_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    commit_file(
        root,
        "crates/dex-cli/src/main.rs",
        "fn main() {}",
        "feat(cli): add main",
    );
    commit_file(
        root,
        "crates/dex-cli/src/main.rs",
        "fn main() { /* fix */ }",
        "fix(cli): handle edge case (#7)",
    );

    dex()
        .args(["context", "sync", "--dir"])
        .arg(root)
        .assert()
        .success();

    let wiki = root.join(".context").join("wiki");
    assert!(wiki.join("INDEX.md").exists(), "INDEX.md should be written");
    assert!(
        root.join(".context").join("USER_MANUAL.md").exists(),
        "USER_MANUAL.md should be written"
    );

    // Two non-merge commits → two node files (plus INDEX.md).
    let nodes: Vec<_> = std::fs::read_dir(&wiki)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.ends_with(".md") && n != "INDEX.md"
        })
        .collect();
    assert_eq!(nodes.len(), 2, "expected one node per commit");

    let index = std::fs::read_to_string(wiki.join("INDEX.md")).unwrap();
    assert!(index.contains("Project Memory"));
    assert!(index.contains("CLI & Interfaces"));
}

#[test]
fn context_sync_is_incremental() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    commit_file(root, "a.rs", "// a", "feat: first");

    dex()
        .args(["context", "sync", "--dir"])
        .arg(root)
        .assert()
        .success();
    commit_file(root, "b.rs", "// b", "feat: second");
    // Second sync should add the new node without erroring on the existing one.
    dex()
        .args(["context", "sync", "--dir"])
        .arg(root)
        .assert()
        .success();

    let wiki = root.join(".context").join("wiki");
    let count = std::fs::read_dir(&wiki)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.ends_with(".md") && n != "INDEX.md"
        })
        .count();
    assert_eq!(count, 2, "incremental run should leave two nodes");
}

#[test]
fn context_sync_outside_git_repo_fails() {
    let dir = tempfile::tempdir().unwrap();
    dex()
        .args(["context", "sync", "--dir"])
        .arg(dir.path())
        .assert()
        .failure();
}

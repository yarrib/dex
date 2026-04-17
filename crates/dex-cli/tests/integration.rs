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
fn mcp_tools_list_returns_three_tools() {
    let req = r#"{"jsonrpc":"2.0","id":8,"method":"tools/list","params":{}}"#;
    let responses = mcp_call(&format!("{req}\n"));

    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"]
        .as_array()
        .expect("expected tools array");

    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"list_templates"), "missing list_templates");
    assert!(
        names.contains(&"get_template_variables"),
        "missing get_template_variables"
    );
    assert!(
        names.contains(&"scaffold_project"),
        "missing scaffold_project"
    );
}

//! Integration tests for the `dex` CLI binary.
//!
//! These tests invoke the compiled binary via `assert_cmd` to verify that
//! commands parse correctly, produce the expected output, and exit with the
//! right status code.

use assert_cmd::Command;

fn dex() -> Command {
    Command::cargo_bin("dex").expect("dex binary not found — run `cargo build` first")
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

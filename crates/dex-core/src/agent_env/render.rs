//! Render `dex.agent-env.toml` into generated project files (devcontainer,
//! auth bootstrap, verify script, CI workflow).

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::agent_env::manifest::AgentEnvManifest;
use crate::error::{AgentEnvError, DexError};
use crate::template::engine::TemplateEngine;

// Built-in agent-env templates are embedded at compile time.
static EMBEDDED_AGENT_ENV: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../agent-env");

#[derive(Debug, Serialize)]
struct RepoToClone {
    slug: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct StepContext {
    name: String,
    run: String,
    always: bool,
}

#[derive(Debug, Serialize)]
struct RenderContext {
    image: String,
    extra_packages: Vec<String>,
    repos_to_clone: Vec<RepoToClone>,
    client_id_env: String,
    client_secret_env: String,
    workspace_host: String,
    profile_name: String,
    catalog: String,
    schema: String,
    steps: Vec<StepContext>,
    cron: Option<String>,
}

fn build_context(manifest: &AgentEnvManifest) -> Result<RenderContext, DexError> {
    let repos_to_clone = manifest
        .base
        .repos
        .iter()
        .filter(|r| r.as_str() != "self")
        .map(|slug| {
            let name = slug.rsplit('/').next().unwrap_or(slug).to_string();
            RepoToClone {
                slug: slug.clone(),
                name,
            }
        })
        .collect();

    let steps = manifest
        .verify
        .steps
        .iter()
        .map(|s| StepContext {
            name: s.name.clone(),
            run: s.run.clone(),
            always: s.always,
        })
        .collect();

    let cron = manifest
        .refresh
        .as_ref()
        .map(|r| r.cron_expression())
        .transpose()?;

    Ok(RenderContext {
        image: manifest.base.image.clone(),
        extra_packages: manifest.base.extra_packages.clone(),
        repos_to_clone,
        client_id_env: manifest.auth.service_principal_env.client_id.clone(),
        client_secret_env: manifest.auth.service_principal_env.client_secret.clone(),
        workspace_host: manifest.auth.workspace_host.clone(),
        profile_name: manifest.auth.workspace.clone(),
        catalog: manifest.auth.scope.catalog.clone(),
        schema: manifest.auth.scope.schema.clone(),
        steps,
        cron,
    })
}

fn render_template(name: &str, context: &RenderContext) -> Result<String, DexError> {
    let file = EMBEDDED_AGENT_ENV.get_file(name).ok_or_else(|| {
        DexError::AgentEnv(AgentEnvError::Invalid(format!(
            "internal error: agent-env template '{name}' is missing from the embedded directory"
        )))
    })?;
    let content = file.contents_utf8().ok_or_else(|| {
        DexError::AgentEnv(AgentEnvError::Invalid(format!(
            "internal error: agent-env template '{name}' is not valid UTF-8"
        )))
    })?;

    let engine = TemplateEngine::new();
    let ctx = minijinja::Value::from_serialize(context);
    engine.render_string(content, &ctx)
}

/// Render `.devcontainer/devcontainer.json` for `project_dir`.
pub fn render_devcontainer_json(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<(PathBuf, String), DexError> {
    let ctx = build_context(manifest)?;
    let content = render_template("devcontainer.json.j2", &ctx)?;
    Ok((
        project_dir.join(".devcontainer").join("devcontainer.json"),
        content,
    ))
}

/// Render `.devcontainer/Dockerfile` for `project_dir`.
pub fn render_dockerfile(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<(PathBuf, String), DexError> {
    let ctx = build_context(manifest)?;
    let content = render_template("Dockerfile.j2", &ctx)?;
    Ok((
        project_dir.join(".devcontainer").join("Dockerfile"),
        content,
    ))
}

/// Render `.devcontainer/auth-bootstrap.sh` for `project_dir`.
pub fn render_auth_bootstrap_sh(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<(PathBuf, String), DexError> {
    let ctx = build_context(manifest)?;
    let content = render_template("auth-bootstrap.sh.j2", &ctx)?;
    Ok((
        project_dir.join(".devcontainer").join("auth-bootstrap.sh"),
        content,
    ))
}

/// Render `.devcontainer/verify.sh` for `project_dir`.
pub fn render_verify_sh(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<(PathBuf, String), DexError> {
    let ctx = build_context(manifest)?;
    let content = render_template("verify.sh.j2", &ctx)?;
    Ok((project_dir.join(".devcontainer").join("verify.sh"), content))
}

/// Render `.github/workflows/agent-env.yml` for `project_dir`.
pub fn render_ci_workflow(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<(PathBuf, String), DexError> {
    let ctx = build_context(manifest)?;
    let content = render_template("agent-env-ci.yml.j2", &ctx)?;
    Ok((
        project_dir
            .join(".github")
            .join("workflows")
            .join("agent-env.yml"),
        content,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_manifest() -> AgentEnvManifest {
        let toml_str = r#"
            version = 1

            [base]
            image = "mcr.microsoft.com/devcontainers/python:3.12"
            extra_packages = ["ripgrep", "jq"]
            repos = ["self", "my-org/shared-libs"]

            [auth]
            provider = "databricks-oauth-m2m"
            workspace = "dev"
            workspace_host = "https://adb-123.4.azuredatabricks.net"

            [auth.service_principal_env]
            client_id = "DATABRICKS_CLIENT_ID"
            client_secret = "DATABRICKS_CLIENT_SECRET"

            [auth.scope]
            catalog = "main"
            schema = "agent_dev"

            [[verify.steps]]
            name = "test"
            run = "pytest -q"
        "#;
        AgentEnvManifest::parse(toml_str).unwrap()
    }

    #[test]
    fn devcontainer_json_contains_env_forwarding_and_repo_clone() {
        let manifest = fixture_manifest();
        let (path, content) = render_devcontainer_json(&manifest, Path::new("/proj")).unwrap();
        assert_eq!(path, Path::new("/proj/.devcontainer/devcontainer.json"));
        assert!(content.contains("DATABRICKS_CLIENT_ID"));
        assert!(content.contains("DATABRICKS_CLIENT_SECRET"));
        assert!(content.contains("my-org/shared-libs"));
        assert!(content.contains("/workspaces/shared-libs"));
        assert!(!content.contains("/workspaces/self"));
        assert!(content.contains("auth-bootstrap.sh"));
    }

    #[test]
    fn dockerfile_pins_image_and_lists_packages() {
        let manifest = fixture_manifest();
        let (path, content) = render_dockerfile(&manifest, Path::new("/proj")).unwrap();
        assert_eq!(path, Path::new("/proj/.devcontainer/Dockerfile"));
        assert!(content.contains("FROM mcr.microsoft.com/devcontainers/python:3.12"));
        assert!(content.contains("ripgrep"));
        assert!(content.contains("jq"));
    }

    #[test]
    fn dockerfile_omits_apt_block_with_no_extra_packages() {
        let toml_str = r#"
            version = 1

            [base]
            image = "mcr.microsoft.com/devcontainers/base:ubuntu"

            [auth]
            provider = "databricks-oauth-m2m"
            workspace = "dev"
            workspace_host = "https://adb-1.azuredatabricks.net"

            [auth.service_principal_env]
            client_id = "ID"
            client_secret = "SECRET"

            [auth.scope]
            catalog = "main"
            schema = "dev"

            [[verify.steps]]
            name = "test"
            run = "pytest"
        "#;
        let manifest = AgentEnvManifest::parse(toml_str).unwrap();
        let (_, content) = render_dockerfile(&manifest, Path::new("/proj")).unwrap();
        assert!(!content.contains("apt-get"));
    }

    fn fixture_manifest_with_refresh_and_always_step() -> AgentEnvManifest {
        let toml_str = r#"
            version = 1

            [base]
            image = "mcr.microsoft.com/devcontainers/python:3.12"
            repos = ["self"]

            [auth]
            provider = "databricks-oauth-m2m"
            workspace = "dev"
            workspace_host = "https://adb-123.4.azuredatabricks.net"

            [auth.service_principal_env]
            client_id = "DATABRICKS_CLIENT_ID"
            client_secret = "DATABRICKS_CLIENT_SECRET"

            [auth.scope]
            catalog = "main"
            schema = "agent_dev"

            [[verify.steps]]
            name = "test"
            run = "pytest -q"

            [[verify.steps]]
            name = "cleanup"
            run = "rm -rf /tmp/agent-scratch"
            always = true

            [refresh]
            schedule = "nightly"
        "#;
        AgentEnvManifest::parse(toml_str).unwrap()
    }

    #[test]
    fn auth_bootstrap_sh_references_configured_env_names_and_host() {
        let manifest = fixture_manifest_with_refresh_and_always_step();
        let (path, content) = render_auth_bootstrap_sh(&manifest, Path::new("/proj")).unwrap();
        assert_eq!(path, Path::new("/proj/.devcontainer/auth-bootstrap.sh"));
        assert!(content.contains("CLIENT_ID_VAR=\"DATABRICKS_CLIENT_ID\""));
        assert!(content.contains("CLIENT_SECRET_VAR=\"DATABRICKS_CLIENT_SECRET\""));
        assert!(content.contains("https://adb-123.4.azuredatabricks.net"));
        assert!(content.contains("PROFILE_NAME=\"dev\""));
        // Never echoes the secret value itself, only the env var name.
        assert!(!content.contains("echo \"$CLIENT_SECRET\""));
    }

    #[test]
    fn verify_sh_runs_always_step_via_trap_and_captures_exit_code() {
        let manifest = fixture_manifest_with_refresh_and_always_step();
        let (path, content) = render_verify_sh(&manifest, Path::new("/proj")).unwrap();
        assert_eq!(path, Path::new("/proj/.devcontainer/verify.sh"));
        assert!(content.contains("trap run_always_steps EXIT"));
        assert!(content.contains("( pytest -q )"));
        assert!(content.contains("( rm -rf /tmp/agent-scratch )"));
        assert!(content.contains("step_code=$?"));
        // The always step's body must live inside run_always_steps, not the
        // ordered non-always block.
        let always_idx = content.find("run_always_steps() {").unwrap();
        let cleanup_idx = content.find("rm -rf /tmp/agent-scratch").unwrap();
        let trap_idx = content.find("trap run_always_steps EXIT").unwrap();
        assert!(always_idx < cleanup_idx && cleanup_idx < trap_idx);
    }

    #[test]
    fn verify_sh_run_value_with_shell_metacharacters_renders_byte_for_byte() {
        let toml_str = r#"
            version = 1

            [base]
            image = "mcr.microsoft.com/devcontainers/base:ubuntu"

            [auth]
            provider = "databricks-oauth-m2m"
            workspace = "dev"
            workspace_host = "https://adb-1.azuredatabricks.net"

            [auth.service_principal_env]
            client_id = "ID"
            client_secret = "SECRET"

            [auth.scope]
            catalog = "main"
            schema = "dev"

            [[verify.steps]]
            name = "weird"
            run = "echo \"hi $(whoami)\" && exit 0"
        "#;
        let manifest = AgentEnvManifest::parse(toml_str).unwrap();
        let (_, content) = render_verify_sh(&manifest, Path::new("/proj")).unwrap();
        assert!(content.contains("( echo \"hi $(whoami)\" && exit 0 )"));
    }

    #[test]
    fn ci_workflow_maps_refresh_schedule_to_cron_trigger() {
        let manifest = fixture_manifest_with_refresh_and_always_step();
        let (path, content) = render_ci_workflow(&manifest, Path::new("/proj")).unwrap();
        assert_eq!(path, Path::new("/proj/.github/workflows/agent-env.yml"));
        assert!(content.contains("cron: \"0 7 * * *\""));
        assert!(content.contains("DATABRICKS_CLIENT_ID: ${{ secrets.DATABRICKS_CLIENT_ID }}"));
        assert!(
            content.contains("DATABRICKS_CLIENT_SECRET: ${{ secrets.DATABRICKS_CLIENT_SECRET }}")
        );
        assert!(content.contains("devcontainers/ci@v0.3"));
        assert!(content.contains("actions/checkout@v4"));
    }

    #[test]
    fn ci_workflow_omits_schedule_trigger_when_refresh_absent() {
        let manifest = fixture_manifest();
        let (_, content) = render_ci_workflow(&manifest, Path::new("/proj")).unwrap();
        assert!(!content.contains("schedule:"));
        assert!(content.contains("workflow_dispatch:"));
    }
}

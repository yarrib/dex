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
struct RenderContext {
    image: String,
    extra_packages: Vec<String>,
    repos_to_clone: Vec<RepoToClone>,
    client_id_env: String,
    client_secret_env: String,
}

fn build_context(manifest: &AgentEnvManifest) -> RenderContext {
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

    RenderContext {
        image: manifest.base.image.clone(),
        extra_packages: manifest.base.extra_packages.clone(),
        repos_to_clone,
        client_id_env: manifest.auth.service_principal_env.client_id.clone(),
        client_secret_env: manifest.auth.service_principal_env.client_secret.clone(),
    }
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
    let ctx = build_context(manifest);
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
    let ctx = build_context(manifest);
    let content = render_template("Dockerfile.j2", &ctx)?;
    Ok((
        project_dir.join(".devcontainer").join("Dockerfile"),
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
}

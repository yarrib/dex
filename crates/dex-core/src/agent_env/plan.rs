//! Plan/apply for `dex agent-env init` — idempotent, diffable file generation.

use std::path::{Path, PathBuf};

use crate::agent_env::manifest::AgentEnvManifest;
use crate::agent_env::render::{
    render_auth_bootstrap_sh, render_ci_workflow, render_devcontainer_json, render_dockerfile,
    render_verify_sh,
};
use crate::error::DexError;

/// Whether a planned file is new, changed, or already up to date on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    Created,
    Updated,
    Unchanged,
}

/// A single generated file and what applying the plan would do to it.
#[derive(Debug)]
pub struct AgentEnvFilePlan {
    pub path: PathBuf,
    pub content: String,
    pub state: FileState,
}

/// The full set of files `dex agent-env init` would write.
#[derive(Debug)]
pub struct AgentEnvPlan {
    pub files: Vec<AgentEnvFilePlan>,
}

/// Render every generated file for `manifest` and classify each against what
/// (if anything) already exists at its destination in `project_dir`.
pub fn plan_agent_env_init(
    manifest: &AgentEnvManifest,
    project_dir: &Path,
) -> Result<AgentEnvPlan, DexError> {
    let rendered = [
        render_devcontainer_json(manifest, project_dir)?,
        render_dockerfile(manifest, project_dir)?,
        render_auth_bootstrap_sh(manifest, project_dir)?,
        render_verify_sh(manifest, project_dir)?,
        render_ci_workflow(manifest, project_dir)?,
    ];

    let files = rendered
        .into_iter()
        .map(|(path, content)| {
            let state = classify(&path, &content);
            AgentEnvFilePlan {
                path,
                content,
                state,
            }
        })
        .collect();

    Ok(AgentEnvPlan { files })
}

fn classify(path: &Path, content: &str) -> FileState {
    match std::fs::read_to_string(path) {
        Ok(existing) if existing == content => FileState::Unchanged,
        Ok(_) => FileState::Updated,
        Err(_) => FileState::Created,
    }
}

/// Write every non-[`FileState::Unchanged`] file in `plan` to disk.
pub fn apply_agent_env_plan(plan: &AgentEnvPlan) -> Result<(), DexError> {
    for file in &plan.files {
        if file.state == FileState::Unchanged {
            continue;
        }
        if let Some(parent) = file.path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(&file.path, &file.content).map_err(|source| DexError::Io {
            path: file.path.clone(),
            source,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_manifest(image: &str, repos: &str) -> AgentEnvManifest {
        let toml_str = format!(
            r#"
            version = 1

            [base]
            image = "{image}"
            repos = [{repos}]

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
        "#
        );
        AgentEnvManifest::parse(&toml_str).unwrap()
    }

    #[test]
    fn first_run_reports_all_created() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = fixture_manifest("mcr.microsoft.com/devcontainers/base:ubuntu", "\"self\"");
        let plan = plan_agent_env_init(&manifest, dir.path()).unwrap();
        assert_eq!(plan.files.len(), 5);
        assert!(plan.files.iter().all(|f| f.state == FileState::Created));
    }

    #[test]
    fn apply_then_replan_reports_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = fixture_manifest("mcr.microsoft.com/devcontainers/base:ubuntu", "\"self\"");
        let plan = plan_agent_env_init(&manifest, dir.path()).unwrap();
        apply_agent_env_plan(&plan).unwrap();

        let replanned = plan_agent_env_init(&manifest, dir.path()).unwrap();
        assert!(
            replanned
                .files
                .iter()
                .all(|f| f.state == FileState::Unchanged),
            "re-running init with an unchanged manifest should report Unchanged for every file"
        );
    }

    #[test]
    fn changing_image_only_updates_dockerfile() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = fixture_manifest("mcr.microsoft.com/devcontainers/base:ubuntu", "\"self\"");
        let plan = plan_agent_env_init(&manifest, dir.path()).unwrap();
        apply_agent_env_plan(&plan).unwrap();

        let changed = fixture_manifest("mcr.microsoft.com/devcontainers/python:3.12", "\"self\"");
        let replanned = plan_agent_env_init(&changed, dir.path()).unwrap();

        for file in &replanned.files {
            if file.path.ends_with("Dockerfile") {
                assert_eq!(file.state, FileState::Updated);
            } else {
                assert_eq!(file.state, FileState::Unchanged);
            }
        }
    }

    #[test]
    fn apply_writes_bytes_matching_plan_content() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = fixture_manifest("mcr.microsoft.com/devcontainers/base:ubuntu", "\"self\"");
        let plan = plan_agent_env_init(&manifest, dir.path()).unwrap();
        apply_agent_env_plan(&plan).unwrap();

        for file in &plan.files {
            let on_disk = std::fs::read_to_string(&file.path).unwrap();
            assert_eq!(on_disk, file.content);
        }
    }

    #[test]
    fn apply_skips_unchanged_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = fixture_manifest("mcr.microsoft.com/devcontainers/base:ubuntu", "\"self\"");
        let plan = plan_agent_env_init(&manifest, dir.path()).unwrap();
        apply_agent_env_plan(&plan).unwrap();

        let dockerfile_path = dir.path().join(".devcontainer").join("Dockerfile");
        let before = std::fs::metadata(&dockerfile_path)
            .unwrap()
            .modified()
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let replanned = plan_agent_env_init(&manifest, dir.path()).unwrap();
        apply_agent_env_plan(&replanned).unwrap();

        let after = std::fs::metadata(&dockerfile_path)
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(before, after, "unchanged file should not be rewritten");
    }
}

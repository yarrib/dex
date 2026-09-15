//! `dex.agent-env.toml` manifest parsing and validation.

use std::path::Path;

use serde::Deserialize;

use crate::error::{AgentEnvError, DexError};

/// Top-level `dex.agent-env.toml` manifest.
#[derive(Debug, Deserialize)]
pub struct AgentEnvManifest {
    pub version: u32,
    pub base: BaseSpec,
    pub auth: AuthSpec,
    pub verify: VerifySpec,
    #[serde(default)]
    pub refresh: Option<RefreshSpec>,
}

/// `[base]` section — the devcontainer base image and repos to clone.
#[derive(Debug, Deserialize)]
pub struct BaseSpec {
    pub image: String,
    #[serde(default)]
    pub extra_packages: Vec<String>,
    #[serde(default = "default_repos")]
    pub repos: Vec<String>,
}

fn default_repos() -> Vec<String> {
    vec!["self".to_string()]
}

/// `[auth]` section — how the agent authenticates to Databricks.
#[derive(Debug, Deserialize)]
pub struct AuthSpec {
    pub provider: String,
    pub workspace: String,
    pub workspace_host: String,
    pub service_principal_env: ServicePrincipalEnvSpec,
    pub scope: ScopeSpec,
}

/// `[auth.service_principal_env]` — env var *names* holding the SP credentials.
#[derive(Debug, Deserialize)]
pub struct ServicePrincipalEnvSpec {
    pub client_id: String,
    pub client_secret: String,
}

/// `[auth.scope]` — the sandbox catalog/schema the SP is scoped to.
#[derive(Debug, Deserialize)]
pub struct ScopeSpec {
    pub catalog: String,
    pub schema: String,
}

/// `[verify]` section — ordered steps the verify script runs.
#[derive(Debug, Deserialize)]
pub struct VerifySpec {
    pub steps: Vec<VerifyStep>,
}

/// A single `[[verify.steps]]` entry.
#[derive(Debug, Deserialize)]
pub struct VerifyStep {
    pub name: String,
    pub run: String,
    #[serde(default)]
    pub always: bool,
}

/// `[refresh]` section — when the CI workflow rebuilds the base image.
#[derive(Debug, Deserialize)]
pub struct RefreshSpec {
    pub schedule: String,
}

impl RefreshSpec {
    /// Map `schedule` to a 5-field cron expression.
    ///
    /// `"nightly"` and `"weekly"` are shorthand for a fixed cron; anything else
    /// is passed through after checking it has 5 whitespace-separated fields.
    pub fn cron_expression(&self) -> Result<String, DexError> {
        match self.schedule.as_str() {
            "nightly" => Ok("0 7 * * *".to_string()),
            "weekly" => Ok("0 7 * * 1".to_string()),
            other => {
                if other.split_whitespace().count() == 5 {
                    Ok(other.to_string())
                } else {
                    Err(DexError::AgentEnv(AgentEnvError::Invalid(
                        "refresh.schedule must be 'nightly', 'weekly', or a 5-field cron expression"
                            .to_string(),
                    )))
                }
            }
        }
    }
}

impl AgentEnvManifest {
    /// Load and parse a `dex.agent-env.toml` file. Does not validate — call
    /// [`AgentEnvManifest::validate`] separately.
    pub fn from_path(path: &Path) -> Result<Self, DexError> {
        let content = std::fs::read_to_string(path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                DexError::AgentEnv(AgentEnvError::ManifestNotFound(path.to_path_buf()))
            } else {
                DexError::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        Self::parse(&content)
    }

    /// Parse a `dex.agent-env.toml` manifest from a string.
    pub fn parse(content: &str) -> Result<Self, DexError> {
        let manifest: AgentEnvManifest = toml::from_str(content).map_err(AgentEnvError::Parse)?;
        Ok(manifest)
    }

    /// Validate the manifest's semantic constraints beyond what serde checks.
    pub fn validate(&self) -> Result<(), DexError> {
        if self.version != 1 {
            return Err(DexError::AgentEnv(AgentEnvError::UnsupportedVersion(
                self.version,
            )));
        }

        if self.base.image.trim().is_empty() {
            return Err(invalid("base.image must not be empty"));
        }
        let repo_re = regex_repo_slug();
        for repo in &self.base.repos {
            if repo != "self" && !repo_re.is_match(repo) {
                return Err(invalid(&format!(
                    "base.repos entry '{repo}' must be 'self' or an 'org/repo' slug"
                )));
            }
        }

        if self.auth.provider != "databricks-oauth-m2m" {
            return Err(DexError::AgentEnv(AgentEnvError::UnsupportedProvider(
                self.auth.provider.clone(),
            )));
        }

        let ident_re = regex_ident();
        let slug_re = regex_slug();

        if !slug_re.is_match(&self.auth.workspace) {
            return Err(invalid(
                "auth.workspace must match ^[A-Za-z0-9_-]+$ (used as a .databrickscfg profile name)",
            ));
        }
        if !self.auth.workspace_host.starts_with("https://")
            || self.auth.workspace_host.contains(char::is_whitespace)
        {
            return Err(invalid(
                "auth.workspace_host must start with https:// and contain no whitespace",
            ));
        }
        if !ident_re.is_match(&self.auth.service_principal_env.client_id) {
            return Err(invalid(
                "auth.service_principal_env.client_id must be a valid shell variable name (^[A-Za-z_][A-Za-z0-9_]*$)",
            ));
        }
        if !ident_re.is_match(&self.auth.service_principal_env.client_secret) {
            return Err(invalid(
                "auth.service_principal_env.client_secret must be a valid shell variable name (^[A-Za-z_][A-Za-z0-9_]*$)",
            ));
        }
        if !slug_re.is_match(&self.auth.scope.catalog) {
            return Err(invalid("auth.scope.catalog must match ^[A-Za-z0-9_-]+$"));
        }
        if !slug_re.is_match(&self.auth.scope.schema) {
            return Err(invalid("auth.scope.schema must match ^[A-Za-z0-9_-]+$"));
        }

        if self.verify.steps.is_empty() {
            return Err(invalid("verify.steps must not be empty"));
        }
        let mut seen_names = std::collections::HashSet::new();
        for step in &self.verify.steps {
            if step.name.trim().is_empty() {
                return Err(invalid("verify.steps[].name must not be empty"));
            }
            if !seen_names.insert(step.name.as_str()) {
                return Err(invalid(&format!(
                    "verify.steps[].name must be unique — duplicate: '{}'",
                    step.name
                )));
            }
            if step.run.trim().is_empty() {
                return Err(invalid(&format!(
                    "verify.steps[].run must not be empty (step '{}')",
                    step.name
                )));
            }
        }

        if let Some(refresh) = &self.refresh {
            refresh.cron_expression()?;
        }

        Ok(())
    }
}

fn invalid(msg: &str) -> DexError {
    DexError::AgentEnv(AgentEnvError::Invalid(msg.to_string()))
}

fn regex_repo_slug() -> regex::Regex {
    regex::Regex::new(r"^[\w.-]+/[\w.-]+$").expect("static regex is valid")
}

fn regex_ident() -> regex::Regex {
    regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("static regex is valid")
}

fn regex_slug() -> regex::Regex {
    regex::Regex::new(r"^[A-Za-z0-9_-]+$").expect("static regex is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> &'static str {
        r#"
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
            name = "lint"
            run = "ruff check ."

            [[verify.steps]]
            name = "cleanup"
            run = "rm -rf /tmp/agent-scratch"
            always = true

            [refresh]
            schedule = "nightly"
        "#
    }

    #[test]
    fn parses_valid_manifest() {
        let manifest = AgentEnvManifest::parse(valid_manifest()).unwrap();
        manifest.validate().unwrap();
        assert_eq!(
            manifest.base.image,
            "mcr.microsoft.com/devcontainers/python:3.12"
        );
        assert_eq!(manifest.base.repos, vec!["self", "my-org/shared-libs"]);
        assert_eq!(manifest.verify.steps.len(), 2);
        assert!(manifest.verify.steps[1].always);
    }

    #[test]
    fn defaults_repos_to_self_when_omitted() {
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
        manifest.validate().unwrap();
        assert_eq!(manifest.base.repos, vec!["self"]);
        assert!(manifest.base.extra_packages.is_empty());
    }

    #[test]
    fn missing_required_field_errors() {
        let toml_str = r#"
            version = 1

            [base]
            image = "img"
        "#;
        let result = AgentEnvManifest::parse(toml_str);
        assert!(
            matches!(result, Err(DexError::AgentEnv(AgentEnvError::Parse(_)))),
            "expected a parse error for missing [auth]/[verify]"
        );
    }

    #[test]
    fn unsupported_version_errors() {
        let toml_str = valid_manifest().replace("version = 1", "version = 2");
        let manifest = AgentEnvManifest::parse(&toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::UnsupportedVersion(2)))
        ));
    }

    #[test]
    fn unknown_provider_errors() {
        let toml_str = valid_manifest().replace(
            r#"provider = "databricks-oauth-m2m""#,
            r#"provider = "azure-generic""#,
        );
        let manifest = AgentEnvManifest::parse(&toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::UnsupportedProvider(_)))
        ));
    }

    #[test]
    fn empty_verify_steps_errors() {
        let toml_str = r#"
            version = 1

            [base]
            image = "img"

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

            [verify]
            steps = []
        "#;
        let manifest = AgentEnvManifest::parse(toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::Invalid(_)))
        ));
    }

    #[test]
    fn duplicate_step_names_error() {
        let toml_str = valid_manifest().replace(r#"name = "cleanup""#, r#"name = "lint""#);
        let manifest = AgentEnvManifest::parse(&toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::Invalid(_)))
        ));
    }

    #[test]
    fn malformed_env_var_name_errors() {
        let toml_str = valid_manifest().replace(
            r#"client_id = "DATABRICKS_CLIENT_ID""#,
            r#"client_id = "not a var name!""#,
        );
        let manifest = AgentEnvManifest::parse(&toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::Invalid(_)))
        ));
    }

    #[test]
    fn bad_refresh_schedule_errors() {
        let toml_str =
            valid_manifest().replace(r#"schedule = "nightly""#, r#"schedule = "sometimes""#);
        let manifest = AgentEnvManifest::parse(&toml_str).unwrap();
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::Invalid(_)))
        ));
    }

    #[test]
    fn cron_expression_mapping() {
        let nightly = RefreshSpec {
            schedule: "nightly".to_string(),
        };
        assert_eq!(nightly.cron_expression().unwrap(), "0 7 * * *");

        let weekly = RefreshSpec {
            schedule: "weekly".to_string(),
        };
        assert_eq!(weekly.cron_expression().unwrap(), "0 7 * * 1");

        let raw = RefreshSpec {
            schedule: "0 3 * * 0".to_string(),
        };
        assert_eq!(raw.cron_expression().unwrap(), "0 3 * * 0");

        let bad = RefreshSpec {
            schedule: "whenever".to_string(),
        };
        assert!(bad.cron_expression().is_err());
    }

    #[test]
    fn from_path_missing_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dex.agent-env.toml");
        let result = AgentEnvManifest::from_path(&path);
        assert!(matches!(
            result,
            Err(DexError::AgentEnv(AgentEnvError::ManifestNotFound(_)))
        ));
    }

    #[test]
    fn from_path_reads_and_parses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dex.agent-env.toml");
        std::fs::write(&path, valid_manifest()).unwrap();
        let manifest = AgentEnvManifest::from_path(&path).unwrap();
        manifest.validate().unwrap();
        assert_eq!(manifest.auth.workspace, "dev");
    }
}

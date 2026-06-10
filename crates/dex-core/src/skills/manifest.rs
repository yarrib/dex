//! Skill pack manifest (`skills.toml`) parsing.

use serde::Deserialize;
use std::path::Path;

use crate::error::{DexError, SkillError};

/// Deserialized skill pack manifest from `skills.toml`.
#[derive(Debug, Deserialize)]
pub struct SkillPackManifest {
    pub pack: PackMeta,
    #[serde(default)]
    pub skills: Vec<SkillSpec>,
}

/// Pack metadata from the `[pack]` section.
#[derive(Debug, Deserialize)]
pub struct PackMeta {
    pub name: String,
    pub description: String,
    pub version: String,
}

/// A single skill entry within a pack.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    pub file: String,
    pub description: String,
}

/// Whether a skill is a slash command, an agent persona, or a native Agent Skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    /// Slash command — installed to `.claude/commands/` or equivalent.
    Command,
    /// Agent persona — installed to `.claude/agents/` or equivalent.
    Agent,
    /// Native Agent Skill (a folder with `SKILL.md`) — installed to
    /// `.claude/skills/<name>/SKILL.md` or equivalent.
    Skill,
}

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillType::Command => write!(f, "command"),
            SkillType::Agent => write!(f, "agent"),
            SkillType::Skill => write!(f, "skill"),
        }
    }
}

impl SkillPackManifest {
    /// Parse a `skills.toml` file from a filesystem path.
    pub fn from_path(path: &Path) -> Result<Self, DexError> {
        let content = std::fs::read_to_string(path).map_err(|source| DexError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&content)
    }

    /// Parse a `skills.toml` from a string.
    pub fn parse(content: &str) -> Result<Self, DexError> {
        toml::from_str(content)
            .map_err(|e| DexError::Skill(SkillError::ManifestParse(e.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml_str = r#"
            [pack]
            name = "test-pack"
            description = "A test skill pack"
            version = "0.1.0"
        "#;
        let manifest = SkillPackManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.pack.name, "test-pack");
        assert!(manifest.skills.is_empty());
    }

    #[test]
    fn parse_full_manifest() {
        let toml_str = r#"
            [pack]
            name = "default"
            description = "General-purpose skills"
            version = "0.1.0"

            [[skills]]
            name = "build"
            type = "command"
            file = "commands/build.md"
            description = "Build the project"

            [[skills]]
            name = "architect"
            type = "agent"
            file = "agents/architect.md"
            description = "Architecture review"
        "#;
        let manifest = SkillPackManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.skills.len(), 2);
        assert_eq!(manifest.skills[0].skill_type, SkillType::Command);
        assert_eq!(manifest.skills[1].skill_type, SkillType::Agent);
    }
}

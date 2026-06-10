//! Install skills from a pack into a project directory.

use std::path::{Path, PathBuf};

use crate::error::{DexError, SkillError};
use crate::skills::manifest::SkillType;
use crate::skills::registry::SkillPack;

/// Where to install skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    /// Claude Code: `.claude/commands/` and `.claude/agents/`
    Claude,
    /// Cursor: `.cursor/rules/<name>.mdc` with YAML frontmatter
    Cursor,
    /// GitHub Copilot: `.github/copilot-instructions.md` (appended sections)
    Copilot,
    /// Generic: `.ai-skills/commands/` and `.ai-skills/agents/`
    Generic,
}

impl InstallTarget {
    /// Parse from a string identifier.
    pub fn parse(s: &str) -> Result<Self, DexError> {
        match s {
            "claude" => Ok(InstallTarget::Claude),
            "cursor" => Ok(InstallTarget::Cursor),
            "copilot" => Ok(InstallTarget::Copilot),
            "generic" => Ok(InstallTarget::Generic),
            other => Err(DexError::Skill(SkillError::InvalidTarget(
                other.to_string(),
            ))),
        }
    }

    /// Human-readable name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            InstallTarget::Claude => "Claude Code (.claude/)",
            InstallTarget::Cursor => "Cursor (.cursor/rules/)",
            InstallTarget::Copilot => "GitHub Copilot (.github/copilot-instructions.md)",
            InstallTarget::Generic => "Generic (.ai-skills/)",
        }
    }

    /// String identifier used in config files.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallTarget::Claude => "claude",
            InstallTarget::Cursor => "cursor",
            InstallTarget::Copilot => "copilot",
            InstallTarget::Generic => "generic",
        }
    }
}

/// Result of installing a skill pack into a project.
#[derive(Debug)]
pub struct InstallResult {
    pub files_written: Vec<PathBuf>,
}

/// Install all skills from a pack into the project at `project_dir`.
pub fn install_skills(
    pack: &SkillPack,
    project_dir: &Path,
    targets: &[InstallTarget],
) -> Result<InstallResult, DexError> {
    let mut files_written = Vec::new();

    for target in targets {
        match target {
            InstallTarget::Claude => {
                install_claude(pack, project_dir, &mut files_written)?;
            }
            InstallTarget::Cursor => {
                install_cursor(pack, project_dir, &mut files_written)?;
            }
            InstallTarget::Copilot => {
                install_copilot(pack, project_dir, &mut files_written)?;
            }
            InstallTarget::Generic => {
                install_generic(pack, project_dir, &mut files_written)?;
            }
        }
    }

    Ok(InstallResult { files_written })
}

// ---------------------------------------------------------------------------
// Target-specific installers
// ---------------------------------------------------------------------------

fn install_claude(
    pack: &SkillPack,
    project_dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<(), DexError> {
    for skill in &pack.manifest.skills {
        let Some(content) = pack.files.get(&skill.file) else {
            continue;
        };

        let (subdir, filename) = match skill.skill_type {
            SkillType::Command => (".claude/commands".to_string(), format!("{}.md", skill.name)),
            SkillType::Agent => (".claude/agents".to_string(), format!("{}.md", skill.name)),
            // Native Agent Skill: a folder named after the skill, holding SKILL.md.
            SkillType::Skill => (
                format!(".claude/skills/{}", skill.name),
                "SKILL.md".to_string(),
            ),
        };

        let dest = project_dir.join(subdir).join(&filename);
        write_file(&dest, content)?;
        written.push(dest);
    }
    Ok(())
}

fn install_cursor(
    pack: &SkillPack,
    project_dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<(), DexError> {
    let rules_dir = project_dir.join(".cursor/rules");

    for skill in &pack.manifest.skills {
        let Some(content) = pack.files.get(&skill.file) else {
            continue;
        };

        // Cursor .mdc format: YAML frontmatter + markdown body. Strip a native
        // skill's own SKILL.md frontmatter so we don't emit two blocks.
        let mdc_content = format!(
            "---\ndescription: {}\n---\n\n{}",
            skill.description,
            skill_body(skill, content)
        );

        let filename = format!("{}.mdc", skill.name);
        let dest = rules_dir.join(&filename);
        write_file(&dest, &mdc_content)?;
        written.push(dest);
    }
    Ok(())
}

fn install_copilot(
    pack: &SkillPack,
    project_dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<(), DexError> {
    let instructions_path = project_dir.join(".github/copilot-instructions.md");

    // Build the section to append (or create).
    let mut section = format!("\n## Skills: {}\n\n", pack.manifest.pack.name);

    for skill in &pack.manifest.skills {
        if let Some(content) = pack.files.get(&skill.file) {
            section.push_str(&format!(
                "### {} ({})\n\n{}\n\n",
                skill.name,
                skill.skill_type,
                skill_body(skill, content)
            ));
        }
    }

    // Read existing content (may not exist yet).
    let existing = if instructions_path.exists() {
        std::fs::read_to_string(&instructions_path).map_err(|source| DexError::Io {
            path: instructions_path.clone(),
            source,
        })?
    } else {
        String::new()
    };

    let new_content = existing + &section;
    write_file(&instructions_path, &new_content)?;
    written.push(instructions_path);
    Ok(())
}

fn install_generic(
    pack: &SkillPack,
    project_dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<(), DexError> {
    for skill in &pack.manifest.skills {
        let Some(content) = pack.files.get(&skill.file) else {
            continue;
        };

        let dest = match skill.skill_type {
            SkillType::Command => project_dir
                .join(".ai-skills/commands")
                .join(format!("{}.md", skill.name)),
            SkillType::Agent => project_dir
                .join(".ai-skills/agents")
                .join(format!("{}.md", skill.name)),
            SkillType::Skill => project_dir
                .join(".ai-skills/skills")
                .join(&skill.name)
                .join("SKILL.md"),
        };
        write_file(&dest, content)?;
        written.push(dest);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Body of a skill for targets that wrap it in their own frontmatter/section.
/// Native Agent Skills carry a `SKILL.md` YAML frontmatter block; strip it so
/// non-Claude targets don't end up with a doubled or stray frontmatter.
fn skill_body<'a>(skill: &crate::skills::manifest::SkillSpec, content: &'a str) -> &'a str {
    if skill.skill_type == SkillType::Skill {
        strip_frontmatter(content)
    } else {
        content
    }
}

/// Drop a leading `---\n … \n---\n` YAML frontmatter block, if present.
fn strip_frontmatter(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        return rest[end + "\n---\n".len()..].trim_start();
    }
    content
}

fn write_file(path: &Path, content: &str) -> Result<(), DexError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(path, content).map_err(|source| DexError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::manifest::{PackMeta, SkillPackManifest, SkillSpec};
    use crate::skills::registry::{SkillPack, SkillSource};
    use std::collections::HashMap;

    fn make_pack(skills: Vec<SkillSpec>, files: HashMap<String, String>) -> SkillPack {
        SkillPack {
            manifest: SkillPackManifest {
                pack: PackMeta {
                    name: "test".to_string(),
                    description: "Test pack".to_string(),
                    version: "0.1.0".to_string(),
                },
                skills,
            },
            source: SkillSource::Embedded,
            files,
        }
    }

    #[test]
    fn install_claude_writes_correct_paths() {
        let dir = tempfile::tempdir().unwrap();

        let skill = SkillSpec {
            name: "build".to_string(),
            skill_type: SkillType::Command,
            file: "commands/build.md".to_string(),
            description: "Build the project".to_string(),
        };

        let mut files = HashMap::new();
        files.insert(
            "commands/build.md".to_string(),
            "Run cargo build.".to_string(),
        );

        let pack = make_pack(vec![skill], files);
        let result = install_skills(&pack, dir.path(), &[InstallTarget::Claude]).unwrap();

        assert_eq!(result.files_written.len(), 1);
        let expected = dir.path().join(".claude/commands/build.md");
        assert!(expected.exists());
        assert_eq!(
            std::fs::read_to_string(&expected).unwrap(),
            "Run cargo build."
        );
    }

    #[test]
    fn install_cursor_adds_frontmatter() {
        let dir = tempfile::tempdir().unwrap();

        let skill = SkillSpec {
            name: "architect".to_string(),
            skill_type: SkillType::Agent,
            file: "agents/architect.md".to_string(),
            description: "Architecture review".to_string(),
        };

        let mut files = HashMap::new();
        files.insert(
            "agents/architect.md".to_string(),
            "You are an architect.".to_string(),
        );

        let pack = make_pack(vec![skill], files);
        install_skills(&pack, dir.path(), &[InstallTarget::Cursor]).unwrap();

        let mdc = dir.path().join(".cursor/rules/architect.mdc");
        assert!(mdc.exists());
        let content = std::fs::read_to_string(&mdc).unwrap();
        assert!(content.starts_with("---\ndescription: Architecture review\n---"));
    }

    #[test]
    fn install_native_skill_writes_skill_md_folder() {
        let dir = tempfile::tempdir().unwrap();

        let skill = SkillSpec {
            name: "project-memory-engine".to_string(),
            skill_type: SkillType::Skill,
            file: "skills/project-memory-engine/SKILL.md".to_string(),
            description: "Build the project-memory graph".to_string(),
        };

        let mut files = HashMap::new();
        files.insert(
            "skills/project-memory-engine/SKILL.md".to_string(),
            "---\nname: project-memory-engine\ndescription: x\n---\n\n# Body\n".to_string(),
        );

        let pack = make_pack(vec![skill], files);
        install_skills(
            &pack,
            dir.path(),
            &[InstallTarget::Claude, InstallTarget::Cursor],
        )
        .unwrap();

        // Claude: native Agent Skill folder, frontmatter preserved.
        let claude = dir
            .path()
            .join(".claude/skills/project-memory-engine/SKILL.md");
        assert!(claude.exists());
        assert!(
            std::fs::read_to_string(&claude)
                .unwrap()
                .contains("name: project-memory-engine")
        );

        // Cursor: single frontmatter block (the skill's own is stripped).
        let mdc = dir.path().join(".cursor/rules/project-memory-engine.mdc");
        let content = std::fs::read_to_string(&mdc).unwrap();
        assert!(content.starts_with("---\ndescription: Build the project-memory graph\n---"));
        assert!(content.contains("# Body"));
        assert!(
            !content.contains("name: project-memory-engine"),
            "inner frontmatter must be stripped"
        );
    }

    #[test]
    fn strip_frontmatter_removes_leading_block() {
        assert_eq!(strip_frontmatter("---\na: b\n---\n\nbody"), "body");
        assert_eq!(strip_frontmatter("no frontmatter"), "no frontmatter");
    }
}

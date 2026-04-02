//! Skill pack discovery and loading from embedded, local, and remote sources.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{RemoteSource, resolve_skill_remote, skills_cache_dir};
use crate::error::{DexError, SkillError};
use crate::skills::manifest::{SkillPackManifest, SkillSpec};

/// Where a skill pack can be loaded from.
#[derive(Debug, Clone)]
pub enum SkillSource {
    /// Built-in packs embedded in the binary at compile time.
    Embedded,
    /// A directory on the filesystem containing skill pack directories.
    Directory(PathBuf),
}

/// Summary entry for a discovered skill pack (for listing).
#[derive(Debug, Clone)]
pub struct SkillPackEntry {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source: SkillSource,
}

/// A fully loaded skill pack — manifest + file contents ready to install.
#[derive(Debug)]
pub struct SkillPack {
    pub manifest: SkillPackManifest,
    pub source: SkillSource,
    /// Map from skill spec file path (e.g. `"commands/build.md"`) to file content.
    pub files: HashMap<String, String>,
}

impl SkillPack {
    /// Return only the command skills.
    #[must_use]
    pub fn commands(&self) -> Vec<&SkillSpec> {
        use crate::skills::manifest::SkillType;
        self.manifest
            .skills
            .iter()
            .filter(|s| s.skill_type == SkillType::Command)
            .collect()
    }

    /// Return only the agent skills.
    #[must_use]
    pub fn agents(&self) -> Vec<&SkillSpec> {
        use crate::skills::manifest::SkillType;
        self.manifest
            .skills
            .iter()
            .filter(|s| s.skill_type == SkillType::Agent)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all available skill packs from all sources.
///
/// Sources are checked in order: embedded → local dir → remote caches.
/// Later sources with the same pack name override earlier ones.
pub fn list_packs(
    extra_dir: Option<&Path>,
    skill_remotes: &[RemoteSource],
) -> Vec<SkillPackEntry> {
    let mut entries: HashMap<String, SkillPackEntry> = HashMap::new();

    // 1. Embedded packs (lowest priority).
    for entry in list_embedded_packs() {
        entries.insert(entry.name.clone(), entry);
    }

    // 2. Local extra directory.
    if let Some(dir) = extra_dir {
        for entry in list_directory_packs(dir, &SkillSource::Directory(dir.to_path_buf())) {
            entries.insert(entry.name.clone(), entry);
        }
    }

    // 3. Remote git repos (pulled to cache).
    for remote in skill_remotes {
        let cache = skills_cache_dir().join(&remote.name);
        if cache.is_dir() {
            let source = SkillSource::Directory(cache.clone());
            for entry in list_directory_packs(&cache, &source) {
                entries.insert(entry.name.clone(), entry);
            }
        }
    }

    let mut result: Vec<SkillPackEntry> = entries.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Load a single skill pack by name, searching all available sources.
pub fn load_pack(
    name: &str,
    extra_dir: Option<&Path>,
    skill_remotes: &[RemoteSource],
) -> Result<SkillPack, DexError> {
    // Try embedded first.
    if let Ok(pack) = load_embedded_pack(name) {
        return Ok(pack);
    }

    // Try local extra directory.
    if let Some(dir) = extra_dir {
        let pack_dir = dir.join(name);
        if pack_dir.is_dir() {
            return load_directory_pack(&pack_dir, SkillSource::Directory(dir.to_path_buf()));
        }
    }

    // Try remote caches.
    for remote in skill_remotes {
        let cache = skills_cache_dir().join(&remote.name);
        let pack_dir = cache.join(name);
        if pack_dir.is_dir() {
            return load_directory_pack(
                &pack_dir,
                SkillSource::Directory(cache),
            );
        }
    }

    Err(DexError::Skill(SkillError::PackNotFound(name.to_string())))
}

/// Resolve and cache remote skill repos, then load a named pack.
pub fn load_pack_with_remote_fetch(
    name: &str,
    extra_dir: Option<&Path>,
    skill_remotes: &[RemoteSource],
    update: bool,
) -> Result<SkillPack, DexError> {
    // Pull remotes to cache first.
    for remote in skill_remotes {
        if let Err(e) = resolve_skill_remote(remote, update) {
            // Non-fatal: stale or missing cache is acceptable.
            let _ = e;
        }
    }
    load_pack(name, extra_dir, skill_remotes)
}

// ---------------------------------------------------------------------------
// Directory-based packs
// ---------------------------------------------------------------------------

fn list_directory_packs(base: &Path, source: &SkillSource) -> Vec<SkillPackEntry> {
    let mut entries = Vec::new();

    let Ok(read_dir) = std::fs::read_dir(base) else {
        return entries;
    };

    for dir_entry in read_dir.flatten() {
        let pack_dir = dir_entry.path();
        let manifest_path = pack_dir.join("skills.toml");
        if manifest_path.is_file()
            && let Ok(manifest) = SkillPackManifest::from_path(&manifest_path)
        {
            entries.push(SkillPackEntry {
                name: manifest.pack.name.clone(),
                description: manifest.pack.description.clone(),
                version: manifest.pack.version.clone(),
                source: source.clone(),
            });
        }
    }

    entries
}

fn load_directory_pack(pack_dir: &Path, source: SkillSource) -> Result<SkillPack, DexError> {
    let manifest_path = pack_dir.join("skills.toml");
    let manifest = SkillPackManifest::from_path(&manifest_path)?;

    let mut files = HashMap::new();
    collect_pack_files(pack_dir, pack_dir, &mut files)?;

    Ok(SkillPack {
        manifest,
        source,
        files,
    })
}

fn collect_pack_files(
    base: &Path,
    dir: &Path,
    files: &mut HashMap<String, String>,
) -> Result<(), DexError> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_pack_files(base, &path, files)?;
        } else if path.is_file()
            && path.extension().is_some_and(|e| e == "md")
            && let Ok(rel) = path.strip_prefix(base)
        {
            let content = std::fs::read_to_string(&path).map_err(|source| DexError::Io {
                path: path.clone(),
                source,
            })?;
            files.insert(rel.to_string_lossy().replace('\\', "/"), content);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Embedded packs
// ---------------------------------------------------------------------------

static EMBEDDED_SKILLS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../skills");

fn list_embedded_packs() -> Vec<SkillPackEntry> {
    let mut entries = Vec::new();

    for pack_dir in EMBEDDED_SKILLS.dirs() {
        let manifest_path = pack_dir.path().join("skills.toml");
        if let Some(file) = EMBEDDED_SKILLS.get_file(&manifest_path)
            && let Some(content) = file.contents_utf8()
            && let Ok(manifest) = SkillPackManifest::parse(content)
        {
            entries.push(SkillPackEntry {
                name: manifest.pack.name.clone(),
                description: manifest.pack.description.clone(),
                version: manifest.pack.version.clone(),
                source: SkillSource::Embedded,
            });
        }
    }

    entries
}

fn load_embedded_pack(name: &str) -> Result<SkillPack, DexError> {
    let pack_dir = EMBEDDED_SKILLS
        .get_dir(name)
        .ok_or_else(|| DexError::Skill(SkillError::PackNotFound(name.to_string())))?;

    let manifest_path = pack_dir.path().join("skills.toml");
    let manifest_file = EMBEDDED_SKILLS
        .get_file(&manifest_path)
        .ok_or_else(|| DexError::Skill(SkillError::PackNotFound(name.to_string())))?;

    let content = manifest_file
        .contents_utf8()
        .ok_or_else(|| DexError::Skill(SkillError::ManifestParse("not valid UTF-8".to_string())))?;

    let manifest = SkillPackManifest::parse(content)?;

    // Collect all .md files from the embedded pack directory.
    let mut files = HashMap::new();
    collect_embedded_files(pack_dir, pack_dir.path(), &mut files);

    Ok(SkillPack {
        manifest,
        source: SkillSource::Embedded,
        files,
    })
}

fn collect_embedded_files(
    dir: &include_dir::Dir<'_>,
    base: &Path,
    files: &mut HashMap<String, String>,
) {
    for file in dir.files() {
        if file.path().extension().is_some_and(|e| e == "md")
            && let (Ok(rel), Some(content)) =
                (file.path().strip_prefix(base), file.contents_utf8())
        {
            files.insert(rel.to_string_lossy().replace('\\', "/"), content.to_string());
        }
    }
    for subdir in dir.dirs() {
        collect_embedded_files(subdir, base, files);
    }
}

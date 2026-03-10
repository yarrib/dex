"""User and project config loading for dex.

Config is loaded from two locations and merged:
  ~/.config/dex/config.toml   user-level config
  ./dex.toml                  project-level config (takes precedence)

Example config:

    [templates]
    dir = "~/my-templates"

    [[templates.remotes]]
    name = "acme"
    url   = "https://github.com/acme/dex-templates"
    ref   = "main"           # optional; defaults to HEAD

    [[templates.remotes]]
    name = "internal"
    url  = "git@github.com:acme/internal-templates.git"
"""

from __future__ import annotations

import subprocess
from dataclasses import dataclass, field
from pathlib import Path

try:
    import tomllib
except ImportError:  # Python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class RemoteSource:
    """A git repository containing dex templates."""

    name: str
    url: str
    ref: str | None = None


@dataclass
class DexConfig:
    """Resolved dex configuration."""

    templates_dir: Path | None = None
    remotes: list[RemoteSource] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

USER_CONFIG_PATH = Path.home() / ".config" / "dex" / "config.toml"
PROJECT_CONFIG_PATH = Path("dex.toml")
REMOTE_CACHE_DIR = Path.home() / ".cache" / "dex" / "templates"
STANDARDS_PATH = Path.home() / ".config" / "dex" / "standards.toml"


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def load_standards(path: Path | None = None) -> dict[str, object]:
    """Load variable pre-fills from a standards file.

    Reads ``~/.config/dex/standards.toml`` by default, or the path provided
    via ``--standards``. Returns a flat dict of variable name → value.
    Variables present here are used directly during ``dex init`` without prompting.

    Example standards.toml::

        author = "yarrib"
        python_version = "3.12"
        org = "acme"
    """
    target = path or STANDARDS_PATH
    if not target.exists():
        return {}
    with open(target, "rb") as f:
        return dict(tomllib.load(f))


def load_config() -> DexConfig:
    """Load and merge user config and project config.

    Project config (./dex.toml) takes precedence over user config.
    """
    user = _parse_config_file(USER_CONFIG_PATH)
    project = _parse_config_file(PROJECT_CONFIG_PATH)
    return _merge(user, project)


def resolve_remote(remote: RemoteSource, *, update: bool = True) -> Path:
    """Return the local cache path for a remote, cloning or pulling as needed.

    If the repo is already cached and ``update`` is False, the existing
    cache is returned without a network call.

    Raises ``RuntimeError`` if clone fails. If pull fails and a cached
    copy exists, a warning is emitted but the stale cache is returned.
    """
    dest = REMOTE_CACHE_DIR / remote.name

    if dest.exists():
        if update:
            _git_pull(dest, remote.ref)
    else:
        REMOTE_CACHE_DIR.mkdir(parents=True, exist_ok=True)
        _git_clone(remote.url, dest, remote.ref)

    return dest


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _parse_config_file(path: Path) -> DexConfig:
    if not path.exists():
        return DexConfig()

    with open(path, "rb") as f:
        data = tomllib.load(f)

    templates = data.get("templates", {})

    templates_dir: Path | None = None
    if raw_dir := templates.get("dir"):
        templates_dir = Path(raw_dir).expanduser().resolve()

    remotes: list[RemoteSource] = []
    for r in templates.get("remotes", []):
        if not isinstance(r, dict) or "name" not in r or "url" not in r:
            continue
        remotes.append(RemoteSource(name=r["name"], url=r["url"], ref=r.get("ref")))

    return DexConfig(templates_dir=templates_dir, remotes=remotes)


def _merge(user: DexConfig, project: DexConfig) -> DexConfig:
    """Project config takes precedence; remotes are additive (project first)."""
    project_names = {r.name for r in project.remotes}
    merged_remotes = project.remotes + [r for r in user.remotes if r.name not in project_names]
    return DexConfig(
        templates_dir=project.templates_dir or user.templates_dir,
        remotes=merged_remotes,
    )


def _git_clone(url: str, dest: Path, ref: str | None) -> None:
    cmd = ["git", "clone", "--depth", "1"]
    if ref:
        cmd += ["--branch", ref]
    cmd += [url, str(dest)]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"Failed to clone template repo '{url}':\n{result.stderr.strip()}")


def _git_pull(dest: Path, ref: str | None) -> None:
    """Pull latest changes. Non-fatal — stale cache is better than no cache."""
    if ref:
        cmds = [
            ["git", "-C", str(dest), "fetch", "--depth", "1", "origin", ref],
            ["git", "-C", str(dest), "checkout", ref],
            ["git", "-C", str(dest), "reset", "--hard", f"origin/{ref}"],
        ]
    else:
        cmds = [["git", "-C", str(dest), "pull", "--ff-only"]]

    for cmd in cmds:
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            # Non-fatal: warn but use cached copy
            import warnings

            warnings.warn(
                f"Could not update template cache at {dest}: {result.stderr.strip()}",
                stacklevel=3,
            )
            return

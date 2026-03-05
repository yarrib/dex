"""Tests for dex.config — user and project config loading."""

from __future__ import annotations

import textwrap
from pathlib import Path

import pytest

from dex.config import DexConfig, RemoteSource, _merge, _parse_config_file


# ---------------------------------------------------------------------------
# _parse_config_file
# ---------------------------------------------------------------------------


def test_parse_empty_file(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text("")
    result = _parse_config_file(cfg)
    assert result.templates_dir is None
    assert result.remotes == []


def test_parse_missing_file(tmp_path: Path) -> None:
    result = _parse_config_file(tmp_path / "nonexistent.toml")
    assert result == DexConfig()


def test_parse_templates_dir(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(textwrap.dedent(f"""
        [templates]
        dir = "{tmp_path}"
    """))
    result = _parse_config_file(cfg)
    assert result.templates_dir == tmp_path.resolve()


def test_parse_remote_https(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(textwrap.dedent("""
        [[templates.remotes]]
        name = "acme"
        url  = "https://github.com/acme/dex-templates"
        ref  = "main"
    """))
    result = _parse_config_file(cfg)
    assert len(result.remotes) == 1
    r = result.remotes[0]
    assert r.name == "acme"
    assert r.url == "https://github.com/acme/dex-templates"
    assert r.ref == "main"


def test_parse_remote_ssh(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(textwrap.dedent("""
        [[templates.remotes]]
        name = "internal"
        url  = "git@github.com:acme/internal-templates.git"
    """))
    result = _parse_config_file(cfg)
    assert result.remotes[0].ref is None


def test_parse_multiple_remotes(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(textwrap.dedent("""
        [[templates.remotes]]
        name = "acme"
        url  = "https://github.com/acme/dex-templates"

        [[templates.remotes]]
        name = "internal"
        url  = "git@github.com:acme/internal.git"
        ref  = "stable"
    """))
    result = _parse_config_file(cfg)
    assert len(result.remotes) == 2
    assert result.remotes[1].ref == "stable"


def test_parse_skips_malformed_remote(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(textwrap.dedent("""
        [[templates.remotes]]
        name = "missing-url"
    """))
    result = _parse_config_file(cfg)
    assert result.remotes == []


# ---------------------------------------------------------------------------
# _merge
# ---------------------------------------------------------------------------


def test_merge_project_overrides_user_dir(tmp_path: Path) -> None:
    user = DexConfig(templates_dir=tmp_path / "user")
    project = DexConfig(templates_dir=tmp_path / "project")
    merged = _merge(user, project)
    assert merged.templates_dir == tmp_path / "project"


def test_merge_falls_back_to_user_dir(tmp_path: Path) -> None:
    user = DexConfig(templates_dir=tmp_path / "user")
    project = DexConfig()
    merged = _merge(user, project)
    assert merged.templates_dir == tmp_path / "user"


def test_merge_remotes_additive_project_first() -> None:
    user_remote = RemoteSource(name="user-remote", url="https://user.example.com")
    project_remote = RemoteSource(name="project-remote", url="https://project.example.com")
    user = DexConfig(remotes=[user_remote])
    project = DexConfig(remotes=[project_remote])
    merged = _merge(user, project)
    assert merged.remotes[0] == project_remote
    assert merged.remotes[1] == user_remote


def test_merge_project_remote_deduplicates_user_remote() -> None:
    shared = RemoteSource(name="acme", url="https://project.example.com")
    user_version = RemoteSource(name="acme", url="https://user.example.com")
    merged = _merge(DexConfig(remotes=[user_version]), DexConfig(remotes=[shared]))
    assert len(merged.remotes) == 1
    assert merged.remotes[0].url == "https://project.example.com"


# ---------------------------------------------------------------------------
# _collect_templates (integration-style, no network)
# ---------------------------------------------------------------------------


def test_collect_templates_embedded_only(monkeypatch: pytest.MonkeyPatch) -> None:
    """With no config, only embedded templates are returned."""
    from dex.config import USER_CONFIG_PATH, PROJECT_CONFIG_PATH

    monkeypatch.setattr("dex.config.USER_CONFIG_PATH", Path("/nonexistent/config.toml"))
    monkeypatch.setattr("dex.config.PROJECT_CONFIG_PATH", Path("/nonexistent/dex.toml"))

    from dex.cli import _collect_templates

    registry = _collect_templates()
    assert "default" in registry
    assert "dabs-package" in registry
    assert registry["default"][0] == "__embedded__"


def test_collect_templates_with_local_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Templates in a local dir override embedded ones with same name, additive otherwise."""
    # Create a minimal custom template
    custom = tmp_path / "my-template"
    custom.mkdir()
    (custom / "template.toml").write_text(
        '[template]\nname = "my-template"\ndescription = "Custom"\nversion = "0.1.0"\nmin_dex_version = "0.1.0"\n'
    )
    (custom / "files").mkdir()

    monkeypatch.setattr("dex.config.USER_CONFIG_PATH", Path("/nonexistent/config.toml"))
    monkeypatch.setattr("dex.config.PROJECT_CONFIG_PATH", Path("/nonexistent/dex.toml"))

    from dex.cli import _collect_templates

    registry = _collect_templates(extra_dir=tmp_path)
    assert "my-template" in registry
    assert registry["my-template"][0] == str(tmp_path)
    # Embedded templates still present
    assert "default" in registry

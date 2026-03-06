.PHONY: build test lint fmt fmt-check dev clean all docs docs-serve help
.PHONY: version bump-patch bump-minor bump-major tag-release _bump-guard

all: lint test

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Development"
	@echo "  dev          uv sync --all-groups + maturin develop"
	@echo "  build        cargo build + maturin develop"
	@echo "  test         cargo test (dex-core) + uv run pytest"
	@echo "  lint         cargo clippy (dex-core) + ruff check"
	@echo "  fmt          cargo fmt + ruff format"
	@echo "  fmt-check    format check only (no writes)"
	@echo "  clean        remove build artifacts"
	@echo ""
	@echo "Docs"
	@echo "  docs         build docs (strict mode)"
	@echo "  docs-serve   build then serve docs at localhost:8000"
	@echo ""
	@echo "Releases"
	@echo "  version      print current version"
	@echo "  bump-patch   bump patch version and commit (open PR, then make tag-release)"
	@echo "  bump-minor   bump minor version and commit (open PR, then make tag-release)"
	@echo "  bump-major   bump major version and commit (open PR, then make tag-release)"
	@echo "  tag-release  tag current HEAD with version in pyproject.toml and push"

dev:
	uv sync --all-groups
	uv run maturin develop --skip-install

build:
	cargo build
	uv run maturin develop --skip-install

test:
	cargo test -p dex-core
	uv run pytest

lint:
	cargo clippy -p dex-core -- -D warnings
	uv run ruff check python/

fmt:
	cargo fmt
	uv run ruff format python/

fmt-check:
	cargo fmt --check
	uv run ruff format --check python/

docs:
	uv sync --group docs
	uv run mkdocs build --strict

docs-serve: docs
	uv run mkdocs serve

clean:
	cargo clean
	rm -rf dist/ .pytest_cache/
	find . \( -name "*.so" -o -name "*.dylib" -o -name "__pycache__" \) -exec rm -rf {} +

# --- Versioning ---

# Print the version currently in pyproject.toml
version:
	@python3 scripts/bump-version.py

# Bump version and commit. Open a PR, merge to main, then run make tag-release.
bump-patch: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py patch))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

bump-minor: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py minor))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

bump-major: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py major))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

# Run this on main after the version bump PR is merged.
# Tags the current HEAD and pushes — triggers the release workflow.
tag-release: _bump-guard
	$(eval VER := $(shell python3 scripts/bump-version.py))
	@git branch --show-current | grep -q '^main$$' || (echo "error: must be on main branch"; exit 1)
	git tag v$(VER)
	git push origin v$(VER)
	@echo "Tagged v$(VER) — release workflow will fire"

_bump-guard:
	@git diff --quiet && git diff --staged --quiet || (echo "error: working tree is dirty"; exit 1)

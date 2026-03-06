.PHONY: build test lint fmt fmt-check dev dev-docs clean all docs docs-serve help
.PHONY: version bump-patch bump-minor bump-major _bump-guard

all: lint test

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Development"
	@echo "  dev          uv sync --group dev + maturin develop"
	@echo "  dev-docs     uv sync --group dev --group docs + maturin develop"
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
	@echo "  bump-patch   bump patch version, tag, and push"
	@echo "  bump-minor   bump minor version, tag, and push"
	@echo "  bump-major   bump major version, tag, and push"

dev:
	uv sync --group dev
	uv run maturin develop

dev-docs:
	uv sync --group dev --group docs
	uv run maturin develop

build:
	cargo build
	uv run maturin develop

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

# Push a tag to trigger the release workflow. The tag is the version source of truth;
# release.yml stamps the version into build artifacts at build time — no commit needed.
bump-patch: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py patch))
	git tag v$(NEW)
	git push origin v$(NEW)
	@echo "Tagged v$(NEW) — release workflow will fire"

bump-minor: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py minor))
	git tag v$(NEW)
	git push origin v$(NEW)
	@echo "Tagged v$(NEW) — release workflow will fire"

bump-major: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py major))
	git tag v$(NEW)
	git push origin v$(NEW)
	@echo "Tagged v$(NEW) — release workflow will fire"

_bump-guard:
	@git diff --quiet && git diff --staged --quiet || (echo "error: working tree is dirty"; exit 1)

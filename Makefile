.PHONY: build test lint fmt fmt-check dev clean all docs docs-serve
.PHONY: version bump-patch bump-minor bump-major _bump-guard

all: lint test

dev:
	uv sync
	maturin develop

build:
	cargo build
	maturin develop

test:
	cargo test
	uv run pytest

lint:
	cargo clippy -- -D warnings
	uv run ruff check python/

fmt:
	cargo fmt
	uv run ruff format python/

fmt-check:
	cargo fmt --check
	uv run ruff format --check python/

docs:
	uv sync --extra docs
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

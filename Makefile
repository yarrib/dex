.PHONY: build test lint fmt fmt-check dev clean all
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

clean:
	cargo clean
	rm -rf dist/ .pytest_cache/
	find . \( -name "*.so" -o -name "*.dylib" -o -name "__pycache__" \) -exec rm -rf {} +

# --- Versioning ---

version:
	@python3 scripts/bump-version.py

bump-patch: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py patch))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to $(NEW)"
	git tag v$(NEW)
	git push origin HEAD v$(NEW)
	@echo "Released v$(NEW)"

bump-minor: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py minor))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to $(NEW)"
	git tag v$(NEW)
	git push origin HEAD v$(NEW)
	@echo "Released v$(NEW)"

bump-major: _bump-guard
	$(eval NEW := $(shell python3 scripts/bump-version.py major))
	git add pyproject.toml crates/dex-core/Cargo.toml crates/dex-py/Cargo.toml
	git commit -m "chore: bump version to $(NEW)"
	git tag v$(NEW)
	git push origin HEAD v$(NEW)
	@echo "Released v$(NEW)"

_bump-guard:
	@git diff --quiet && git diff --staged --quiet || (echo "error: working tree is dirty"; exit 1)
	@git rev-parse --abbrev-ref HEAD | grep -q '^main$$' || (echo "error: not on main branch"; exit 1)

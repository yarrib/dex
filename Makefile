.PHONY: build test lint fmt fmt-check clean all docs docs-serve help
.PHONY: version bump-patch bump-minor bump-major tag-release _bump-guard

all: lint test

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Development"
	@echo "  build        cargo build"
	@echo "  test         cargo test"
	@echo "  lint         cargo clippy -- -D warnings"
	@echo "  fmt          cargo fmt"
	@echo "  fmt-check    format check only (no writes)"
	@echo "  clean        remove build artifacts"
	@echo ""
	@echo "Docs"
	@echo "  docs         build docs"
	@echo "  docs-serve   serve docs at localhost:3000"
	@echo ""
	@echo "Releases"
	@echo "  version      print current version"
	@echo "  bump-patch   bump patch version and commit (open PR, then make tag-release)"
	@echo "  bump-minor   bump minor version and commit (open PR, then make tag-release)"
	@echo "  bump-major   bump major version and commit (open PR, then make tag-release)"
	@echo "  tag-release  tag current HEAD with version from Cargo.toml and push"

build:
	cargo build

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

docs:
	mdbook build

docs-serve:
	mdbook serve

clean:
	cargo clean

# --- Versioning ---

version:
	@bash scripts/bump-version.sh

bump-patch: _bump-guard
	$(eval NEW := $(shell bash scripts/bump-version.sh patch))
	git add crates/dex-core/Cargo.toml crates/dex-cli/Cargo.toml Cargo.lock
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

bump-minor: _bump-guard
	$(eval NEW := $(shell bash scripts/bump-version.sh minor))
	git add crates/dex-core/Cargo.toml crates/dex-cli/Cargo.toml Cargo.lock
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

bump-major: _bump-guard
	$(eval NEW := $(shell bash scripts/bump-version.sh major))
	git add crates/dex-core/Cargo.toml crates/dex-cli/Cargo.toml Cargo.lock
	git commit -m "chore: bump version to v$(NEW)"
	@echo "Version bumped to v$(NEW). Push a PR, merge to main, then: make tag-release"

# Run this on main after the version bump PR is merged.
# Tags the current HEAD and pushes — triggers the release workflow.
tag-release: _bump-guard
	$(eval VER := $(shell bash scripts/bump-version.sh))
	@git branch --show-current | grep -q '^main$$' || (echo "error: must be on main branch"; exit 1)
	git tag v$(VER)
	git push origin v$(VER)
	@echo "Tagged v$(VER) — release workflow will fire"

_bump-guard:
	@git diff --quiet && git diff --staged --quiet || (echo "error: working tree is dirty"; exit 1)

.PHONY: build test lint fmt fmt-check clean all docs docs-serve docs-install help
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
	@echo "  docs-install install mdbook and git-cliff via cargo"
	@echo "  docs         build docs (generates changelog if git-cliff is available)"
	@echo "  docs-serve   serve docs at localhost:3000 and open browser"
	@echo ""
	@echo "Releases"
	@echo "  version      print current version"
	@echo "  check-version verify version isn't behind the latest tag (CI enforces this)"
	@echo "  bump-patch   branch off main, bump patch, push, open release PR"
	@echo "  bump-minor   branch off main, bump minor, push, open release PR"
	@echo "  bump-major   branch off main, bump major, push, open release PR"
	@echo "               (merging the PR auto-tags + releases via tag-on-merge.yml)"
	@echo "  tag-release  manual fallback: tag main HEAD with the Cargo.toml version and push"

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

docs-install:
	cargo install mdbook
	cargo install git-cliff

docs:
	@if command -v git-cliff >/dev/null 2>&1; then \
		git-cliff --output docs/changelog.md; \
	fi
	mdbook build

docs-serve:
	@if command -v git-cliff >/dev/null 2>&1; then \
		git-cliff --output docs/changelog.md; \
	fi
	mdbook serve --open

clean:
	cargo clean

# --- Versioning ---

version:
	@bash scripts/bump-version.sh

# Verify the version isn't behind the latest tag (also enforced in CI).
check-version:
	@bash scripts/check-version.sh

# Each bump target branches off main, bumps the version, pushes, and opens a PR.
# Merging that PR fires .github/workflows/tag-on-merge.yml, which tags the
# release and dispatches release.yml — no manual tagging needed.
bump-patch:
	@bash scripts/release.sh patch

bump-minor:
	@bash scripts/release.sh minor

bump-major:
	@bash scripts/release.sh major

# Manual fallback. The normal path auto-tags on PR merge (tag-on-merge.yml);
# use this only to tag main HEAD by hand (e.g. if that workflow is disabled).
# Tags the current HEAD (annotated with changelog) and pushes — triggers the release workflow.
# Falls back to a lightweight tag if git-cliff isn't installed; CI will promote it.
tag-release: _bump-guard
	$(eval VER := $(shell bash scripts/bump-version.sh))
	@git branch --show-current | grep -q '^main$$' || (echo "error: must be on main branch"; exit 1)
	@if command -v git-cliff >/dev/null 2>&1; then \
		printf 'Release v$(VER)\n\n' > /tmp/dex-tag-msg.md; \
		git-cliff --unreleased --tag v$(VER) --strip header >> /tmp/dex-tag-msg.md; \
		git tag -a -F /tmp/dex-tag-msg.md v$(VER); \
		rm -f /tmp/dex-tag-msg.md; \
	else \
		echo "warn: git-cliff not installed — creating lightweight tag (CI will promote)"; \
		git tag v$(VER); \
	fi
	git push origin v$(VER)
	@echo "Tagged v$(VER) — release workflow will fire"

_bump-guard:
	@git diff --quiet && git diff --staged --quiet || (echo "error: working tree is dirty"; exit 1)

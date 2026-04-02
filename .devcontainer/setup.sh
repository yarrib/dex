#!/usr/bin/env bash
# .devcontainer/setup.sh — post-create setup for the dex devcontainer.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"

echo "==> Installing uv"
if ! command -v uv >/dev/null 2>&1; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="${HOME}/.cargo/bin:${HOME}/.local/bin:${PATH}"
fi

echo "==> Installing Python dependencies"
uv sync

echo "==> Building Rust extension (maturin develop)"
uv run maturin develop

echo "==> dex dev environment ready"
echo ""

# ai-dev-kit skill setup — runs last so it can use dex itself
bash "${REPO_ROOT}/scripts/setup_dev_kit.sh"

#!/usr/bin/env bash
# scripts/setup_dev_kit.sh
#
# Installs Databricks AI Dev Kit into the project with profile-based skill selection.
# Skills are vendored into the project so every assistant gets the right context.
#
# Usage:
#   bash scripts/setup_dev_kit.sh
#
# Configuration (env vars or .devcontainer/config.toml [ai] section):
#   DEVKIT_PROFILE   — skill profile: ai-ml-engineer | data-engineer | analyst | app-developer
#   DEVKIT_TOOL      — AI assistant:  claude | cursor | copilot | codex | gemini | all
#   AI_DEV_KIT_REF   — git ref to install from (default: main)
#
# All other DEVKIT_* vars are passed through to ai-dev-kit's install.sh unchanged.

set -euo pipefail

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
CONFIG_FILE="${REPO_ROOT}/.devcontainer/config.toml"

# ai-dev-kit source
AI_DEV_KIT_REF="${AI_DEV_KIT_REF:-main}"
AI_DEV_KIT_RAW_BASE="https://raw.githubusercontent.com/databricks-solutions/ai-dev-kit/${AI_DEV_KIT_REF}"

# Vendored skills land here — committed to the repo so the assistant always has context
VENDOR_SKILLS_DIR="${VENDOR_SKILLS_DIR:-${REPO_ROOT}/skills/databricks}"
SKILLS_DEFAULT_DIR="${SKILLS_DEFAULT_DIR:-${REPO_ROOT}/.skills}"

# ---------------------------------------------------------------------------
# Resolve profile + tool
# ---------------------------------------------------------------------------

DEVKIT_PROFILE="${DEVKIT_PROFILE:-}"
DEVKIT_TOOL="${DEVKIT_TOOL:-}"

# 1. Try .devcontainer/config.toml
if [ -z "${DEVKIT_PROFILE}" ] || [ -z "${DEVKIT_TOOL}" ]; then
  if [ -f "${CONFIG_FILE}" ] && command -v python3 >/dev/null 2>&1; then
    _parsed="$(python3 - <<'EOF'
import sys, os
path = os.environ.get("CONFIG_FILE", "")
try:
    try:
        import tomllib
    except ImportError:
        import tomli as tomllib
    with open(path, "rb") as f:
        data = tomllib.load(f)
    ai = data.get("ai", {})
    print(ai.get("profile", ""))
    print(ai.get("assistant", ""))
except Exception:
    print("")
    print("")
EOF
)" || true
    _cfg_profile="$(echo "${_parsed}" | sed -n '1p')"
    _cfg_tool="$(echo "${_parsed}" | sed -n '2p')"
    DEVKIT_PROFILE="${DEVKIT_PROFILE:-${_cfg_profile}}"
    DEVKIT_TOOL="${DEVKIT_TOOL:-${_cfg_tool}}"
  fi
fi

# 2. Interactive fallback (skip if non-interactive)
if [ -z "${DEVKIT_PROFILE}" ]; then
  if [ -t 0 ]; then
    echo ""
    echo "Select a skill profile:"
    echo "  1) ai-ml-engineer  — agents, MLflow, vector search, model serving"
    echo "  2) data-engineer   — pipelines, DLT, Unity Catalog, Iceberg"
    echo "  3) analyst         — AI/BI dashboards, Genie, SQL"
    echo "  4) app-developer   — Databricks Apps, FastAPI, Streamlit"
    printf "Profile [1-4, default 1]: "
    read -r _choice
    case "${_choice}" in
      2) DEVKIT_PROFILE="data-engineer" ;;
      3) DEVKIT_PROFILE="analyst" ;;
      4) DEVKIT_PROFILE="app-developer" ;;
      *) DEVKIT_PROFILE="ai-ml-engineer" ;;
    esac
  else
    echo "==> Non-interactive: defaulting to profile=ai-ml-engineer"
    DEVKIT_PROFILE="ai-ml-engineer"
  fi
fi

if [ -z "${DEVKIT_TOOL}" ]; then
  if [ -t 0 ]; then
    echo ""
    echo "Select your AI assistant:"
    echo "  1) claude   2) cursor   3) copilot   4) codex   5) gemini   6) all"
    printf "Assistant [1-6, default 1]: "
    read -r _choice
    case "${_choice}" in
      2) DEVKIT_TOOL="cursor" ;;
      3) DEVKIT_TOOL="copilot" ;;
      4) DEVKIT_TOOL="codex" ;;
      5) DEVKIT_TOOL="gemini" ;;
      6) DEVKIT_TOOL="all" ;;
      *) DEVKIT_TOOL="claude" ;;
    esac
  else
    echo "==> Non-interactive: defaulting to tool=claude"
    DEVKIT_TOOL="claude"
  fi
fi

# ---------------------------------------------------------------------------
# Export ai-dev-kit env vars
# ---------------------------------------------------------------------------

export AIDEVKIT_HOME="${AIDEVKIT_HOME:-${REPO_ROOT}/.ai-dev-kit}"
export DEVKIT_SCOPE="${DEVKIT_SCOPE:-project}"
export DEVKIT_SKILLS_PROFILE="${DEVKIT_PROFILE}"
export DEVKIT_TOOLS="${DEVKIT_TOOL}"
export DEVKIT_SILENT="${DEVKIT_SILENT:-1}"
export DEVKIT_FORCE="${DEVKIT_FORCE:-}"

# Pass through any DATABRICKS_* auth vars if set
# DATABRICKS_TOKEN, DATABRICKS_CONFIG_PROFILE — used by the MCP server at runtime, not install

echo ""
echo "==> Installing Databricks AI Dev Kit"
echo "    ref:     ${AI_DEV_KIT_REF}"
echo "    profile: ${DEVKIT_PROFILE}"
echo "    tool:    ${DEVKIT_TOOL}"
echo "    scope:   ${DEVKIT_SCOPE}"
echo ""

# ---------------------------------------------------------------------------
# Install ai-dev-kit
# ---------------------------------------------------------------------------

bash <(curl -fsSL "${AI_DEV_KIT_RAW_BASE}/install.sh")

# ---------------------------------------------------------------------------
# Vendor skills into skills/databricks/
#
# Copies installed skill markdown files into the versioned skills/databricks/
# directory so `dex skills sync` can distribute them to any assistant via
# the standard dex skills machinery.  This makes ai-dev-kit a *source* for
# the dex skills system rather than a parallel install path.
# ---------------------------------------------------------------------------

_vendor_from() {
  local src_dir="$1"
  local dest_dir="$2"
  if [ -d "${src_dir}" ]; then
    mkdir -p "${dest_dir}"
    find "${src_dir}" -name "*.md" | while IFS= read -r f; do
      _rel="${f#${src_dir}/}"
      _dest="${dest_dir}/${_rel}"
      mkdir -p "$(dirname "${_dest}")"
      cp "${f}" "${_dest}"
    done
    echo "==> Vendored $(find "${src_dir}" -name '*.md' | wc -l | tr -d ' ') skills → ${dest_dir#${REPO_ROOT}/}"
  fi
}

# ai-dev-kit places project-scope skills in tool-specific dirs;
# we unify them into skills/databricks/ for dex to manage
_AIDEVKIT_SKILLS="${AIDEVKIT_HOME}/databricks-skills"
if [ -d "${_AIDEVKIT_SKILLS}" ]; then
  _vendor_from "${_AIDEVKIT_SKILLS}" "${VENDOR_SKILLS_DIR}"
fi

# MLflow skills (downloaded by ai-dev-kit separately)
_MLFLOW_SKILLS="${AIDEVKIT_HOME}/mlflow-skills"
if [ -d "${_MLFLOW_SKILLS}" ]; then
  _vendor_from "${_MLFLOW_SKILLS}" "${VENDOR_SKILLS_DIR}/mlflow"
fi

echo ""
echo "==> Done. Run 'dex skills sync' to install skills for your assistant."

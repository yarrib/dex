#!/usr/bin/env sh
# dex installer
# Usage: curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
# Options:
#   --platform linux   Force Linux target
#   --platform macos   Force macOS target
set -eu

REPO="yarrib/dex"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"
GITHUB_RELEASES="https://github.com/${REPO}/releases/download"
INSTALL_DIR="${HOME}/.local/bin"

# --- Parse args ---

PLATFORM_OVERRIDE=""
for arg in "$@"; do
  case "${arg}" in
    linux|macos) PLATFORM_OVERRIDE="${arg}" ;;
    --platform=linux) PLATFORM_OVERRIDE="linux" ;;
    --platform=macos) PLATFORM_OVERRIDE="macos" ;;
  esac
done

# --- OS / arch detection ---

OS="$(uname -s)"
ARCH="$(uname -m)"

EFFECTIVE_OS="${PLATFORM_OVERRIDE:-}"
if [ -z "${EFFECTIVE_OS}" ]; then
  case "${OS}" in
    Linux)  EFFECTIVE_OS="linux" ;;
    Darwin) EFFECTIVE_OS="macos" ;;
    *)
      echo "Unsupported OS: ${OS}"
      echo "Download manually from https://github.com/${REPO}/releases"
      exit 1
      ;;
  esac
fi

case "${EFFECTIVE_OS}" in
  linux)
    case "${ARCH}" in
      x86_64)        TARGET="linux-x86_64" ;;
      aarch64|arm64) TARGET="linux-aarch64" ;;
      *)
        echo "Unsupported Linux architecture: ${ARCH}"
        echo "Download manually from https://github.com/${REPO}/releases"
        exit 1
        ;;
    esac
    ;;
  macos)
    case "${ARCH}" in
      arm64)  TARGET="macos-aarch64" ;;
      x86_64) TARGET="macos-x86_64" ;;
      *)
        echo "Unsupported macOS architecture: ${ARCH}"
        echo "Download manually from https://github.com/${REPO}/releases"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported platform: ${EFFECTIVE_OS}. Use --platform linux or --platform macos."
    exit 1
    ;;
esac

# --- Fetch latest release tag ---

echo "Fetching latest dex release..."

if command -v curl >/dev/null 2>&1; then
  TAG="$(curl -sSf "${GITHUB_API}" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
elif command -v wget >/dev/null 2>&1; then
  TAG="$(wget -qO- "${GITHUB_API}" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
else
  echo "Error: curl or wget is required."
  exit 1
fi

if [ -z "${TAG}" ]; then
  echo "Error: could not determine latest release tag."
  echo "Check https://github.com/${REPO}/releases"
  exit 1
fi

# --- Download binary ---

BINARY_NAME="dex-${TARGET}"
BINARY_URL="${GITHUB_RELEASES}/${TAG}/${BINARY_NAME}"

echo "Installing dex ${TAG} for ${TARGET}..."

TMP_FILE="$(mktemp)"
trap 'rm -f "${TMP_FILE}"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -sSfL "${BINARY_URL}" -o "${TMP_FILE}"
else
  wget -qO "${TMP_FILE}" "${BINARY_URL}"
fi

# --- Install ---

mkdir -p "${INSTALL_DIR}"
chmod +x "${TMP_FILE}"
mv "${TMP_FILE}" "${INSTALL_DIR}/dex"

# --- Verify ---

echo ""
echo "dex installed to ${INSTALL_DIR}/dex"

# Warn if INSTALL_DIR is not on PATH
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "Note: ${INSTALL_DIR} is not on your PATH."
    echo "Add this to your shell profile:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

"${INSTALL_DIR}/dex" --version

# --- Offer to wire up the MCP server ---

DEX_BIN="${INSTALL_DIR}/dex"

echo ""
echo "dex ships an MCP server (dex mcp serve) for AI coding assistants"
echo "(Claude Code, Cursor, VS Code/Copilot, Codex, Zed, Antigravity, ...)."

# Only prompt when a terminal is attached. With \`curl ... | sh\` stdin is the
# pipe, so read from /dev/tty if it's available; otherwise just print the hint.
if [ -r /dev/tty ]; then
  printf "Wire it into your editors now? [y/N] "
  read -r REPLY < /dev/tty || REPLY=""
  case "${REPLY}" in
    [yY] | [yY][eE][sS])
      # Pass the absolute path so GUI clients don't depend on PATH.
      "${DEX_BIN}" mcp install --command "${DEX_BIN}" < /dev/tty || true
      ;;
    *)
      echo "Skipped. Wire it up later with: dex mcp install"
      ;;
  esac
else
  echo "Wire it up with: dex mcp install"
fi

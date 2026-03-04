#!/usr/bin/env sh
# dex installer
# Usage: curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
# Options:
#   --platform linux   Force Linux wheel (default on Linux)
#   --platform macos   Force macOS wheel (default on macOS)
set -eu

REPO="yarrib/dex"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"
GITHUB_RELEASES="https://github.com/${REPO}/releases/download"

# --- Parse args ---

PLATFORM_OVERRIDE=""
for arg in "$@"; do
  case "${arg}" in
    --platform) ;;
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
    Linux)   EFFECTIVE_OS="linux" ;;
    Darwin)  EFFECTIVE_OS="macos" ;;
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
      x86_64)       PLATFORM="manylinux_2_17_x86_64.manylinux2014_x86_64" ;;
      aarch64|arm64) PLATFORM="manylinux_2_17_aarch64.manylinux2014_aarch64" ;;
      *)
        echo "Unsupported Linux architecture: ${ARCH}"
        echo "Download manually from https://github.com/${REPO}/releases"
        exit 1
        ;;
    esac
    ;;
  macos)
    case "${ARCH}" in
      arm64)  PLATFORM="macosx_11_0_arm64" ;;
      x86_64) PLATFORM="macosx_10_12_x86_64" ;;
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

# Strip leading 'v' for the version component in the wheel filename
VERSION="${TAG#v}"

# --- Detect system Python, build PYTAG fallback chain (newest first) ---

DETECTED_PY="$(python3 --version 2>/dev/null | sed 's/Python 3\.\([0-9]*\).*/cp3\1/' || true)"

# Build deduped list: detected version first, then cp313 → cp312 → cp311
PYTAGS="${DETECTED_PY}"
for v in cp313 cp312 cp311; do
  case " ${PYTAGS} " in *" ${v} "*) ;; *) PYTAGS="${PYTAGS} ${v}" ;; esac
done

# --- Find first available wheel ---

WHEEL=""
WHEEL_URL=""
for PYTAG in ${PYTAGS}; do
  CANDIDATE="dex-${VERSION}-${PYTAG}-${PYTAG}-${PLATFORM}.whl"
  CANDIDATE_URL="${GITHUB_RELEASES}/${TAG}/${CANDIDATE}"

  if command -v curl >/dev/null 2>&1; then
    HTTP_STATUS="$(curl -sSo /dev/null -w "%{http_code}" -I "${CANDIDATE_URL}")"
    if [ "${HTTP_STATUS}" = "200" ] || [ "${HTTP_STATUS}" = "302" ]; then
      WHEEL="${CANDIDATE}"
      WHEEL_URL="${CANDIDATE_URL}"
      break
    fi
  else
    # wget: no easy HEAD check — take first candidate, uv will fail if it's wrong
    WHEEL="${CANDIDATE}"
    WHEEL_URL="${CANDIDATE_URL}"
    break
  fi
done

if [ -z "${WHEEL}" ]; then
  echo "No compatible wheel found for ${PLATFORM} (tried:${PYTAGS})."
  echo "Download manually from https://github.com/${REPO}/releases/tag/${TAG}"
  exit 1
fi

echo "Installing dex ${TAG} (${WHEEL})..."

# --- Ensure uv is available ---

if ! command -v uv >/dev/null 2>&1; then
  echo "uv not found — installing uv first..."
  if command -v curl >/dev/null 2>&1; then
    curl -LsSf https://astral.sh/uv/install.sh | sh
  else
    wget -qO- https://astral.sh/uv/install.sh | sh
  fi
  export PATH="${HOME}/.local/bin:${PATH}"
fi

# --- Install dex ---

uv tool install "${WHEEL_URL}"

echo ""
echo "dex installed successfully!"
dex --version

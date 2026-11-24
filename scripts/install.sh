#!/usr/bin/env bash
# LLVM IR Studio (llvm-ir-ls) Installer
# Installs the pre-compiled llvm-ir-ls binary to ~/.local/bin or a custom directory.
set -euo pipefail

REPO="SohamTilekar/llvm-ir-studio"
BINARY_NAME="llvm-ir-ls"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

log_info() { echo -e "${BLUE}==>${RESET} ${BOLD}$1${RESET}"; }
log_success() { echo -e "${GREEN}==>${RESET} ${BOLD}$1${RESET}"; }
log_warn() { echo -e "${YELLOW}WARNING:${RESET} $1"; }
log_err() { echo -e "${RED}ERROR:${RESET} $1" >&2; }

echo -e "${BOLD}LLVM IR Studio — Language Server Installer (${BINARY_NAME})${RESET}"
echo "--------------------------------------------------------"

# 1. Detect OS & Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)
    OS_TARGET="unknown-linux-gnu"
    ;;
  Darwin)
    OS_TARGET="apple-darwin"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    log_err "Windows detected. Please use install.ps1 in PowerShell, or run: cargo install llvm-ir-ls"
    exit 1
    ;;
  *)
    log_err "Unsupported operating system: ${OS}"
    exit 1
    ;;
esac

case "${ARCH}" in
  x86_64|amd64)
    ARCH_TARGET="x86_64"
    ;;
  aarch64|arm64)
    ARCH_TARGET="aarch64"
    ;;
  *)
    log_err "Unsupported CPU architecture: ${ARCH}"
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
log_info "Detected platform: ${BOLD}${TARGET}${RESET}"

# 2. Determine target version / tag
VERSION="${VERSION:-}"
if [ -z "${VERSION}" ]; then
  log_info "Checking latest release from GitHub..."
  LATEST_TAG=$(curl -sSL -H "Accept: application/vnd.github+json" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -m1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' || true)
  if [ -z "${LATEST_TAG}" ]; then
    # Fallback to redirect resolution
    LATEST_TAG=$(curl -sI "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -i '^location:' | sed -E 's/.*tag\/(.*)/\1/' | tr -d '\r\n' || true)
  fi
  VERSION="${LATEST_TAG:-v0.1.0}"
fi

# Ensure version has 'v' prefix for release tag matching
case "${VERSION}" in
  v*) TAG="${VERSION}" ;;
  *) TAG="v${VERSION}" ;;
esac

ARCHIVE_NAME="${BINARY_NAME}-${TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE_NAME}"

log_info "Downloading ${BINARY_NAME} (${TAG}) for ${TARGET}..."

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

HTTP_CODE=0
if command -v curl >/dev/null 2>&1; then
  HTTP_CODE=$(curl -sSL -w "%{http_code}" -o "${TMP_DIR}/${ARCHIVE_NAME}" "${DOWNLOAD_URL}" || true)
elif command -v wget >/dev/null 2>&1; then
  if wget -q -O "${TMP_DIR}/${ARCHIVE_NAME}" "${DOWNLOAD_URL}"; then
    HTTP_CODE=200
  else
    HTTP_CODE=404
  fi
else
  log_err "Neither curl nor wget is installed."
  exit 1
fi

DOWNLOAD_SUCCESS=false
if [ "${HTTP_CODE}" = "200" ] && [ -s "${TMP_DIR}/${ARCHIVE_NAME}" ]; then
  if tar -tzf "${TMP_DIR}/${ARCHIVE_NAME}" >/dev/null 2>&1; then
    tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}"
    DOWNLOAD_SUCCESS=true
  fi
fi

if [ "${DOWNLOAD_SUCCESS}" = false ]; then
  log_warn "Could not download pre-built release binary from: ${DOWNLOAD_URL}"
  if command -v cargo >/dev/null 2>&1; then
    log_info "Rust toolchain (cargo) detected. Building and installing from source..."
    if [ -f "crates/llvm-ir-ls/Cargo.toml" ]; then
      cargo install --path crates/llvm-ir-ls --root "${HOME}/.local"
    else
      cargo install --git "https://github.com/${REPO}.git" llvm-ir-ls --root "${HOME}/.local"
    fi
    log_success "Successfully built and installed llvm-ir-ls via cargo!"
    exit 0
  else
    log_err "Failed to download binary and cargo is not available."
    log_err "Please check releases at https://github.com/${REPO}/releases or install Rust."
    exit 1
  fi
fi

# 3. Install binary
mkdir -p "${INSTALL_DIR}"
mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

log_success "${BINARY_NAME} installed successfully to: ${INSTALL_DIR}/${BINARY_NAME}"

# 4. Verify installation & PATH check
if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  log_warn "${INSTALL_DIR} is not in your system \$PATH."
  echo "Add it to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
  echo -e "    ${BOLD}export PATH=\"${INSTALL_DIR}:\$PATH\"${RESET}"
  echo ""
fi

if command -v "${INSTALL_DIR}/${BINARY_NAME}" >/dev/null 2>&1; then
  VERSION_OUT="$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null || true)"
  log_success "Verified: ${VERSION_OUT}"
fi

echo ""
echo -e "Next steps:"
echo -e "  Run the editor setup script to configure your editors automatically:"
echo -e "    ${BOLD}./setup-editor.sh${RESET}  (or: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/setup-editor.sh | bash)"
echo "--------------------------------------------------------"

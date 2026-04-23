#!/bin/sh
set -e

# Repository info
OWNER="V-VX"
REPO="iqos_cli"
GITHUB_API_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

info() {
    echo "${GREEN}INFO:${NC} $1"
}

error() {
    echo "${RED}ERROR:${NC} $1" >&2
    exit 1
}

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)
        case "${ARCH}" in
            x86_64)
                ASSET_PACKAGE="linux-x86_64"
                ;;
            *)
                error "Unsupported architecture for Linux: ${ARCH}"
                ;;
        esac
        ;;
    Darwin)
        # macOS has a universal binary
        ASSET_PACKAGE="macos-universal"
        ;;
    *)
        error "Unsupported OS: ${OS}"
        ;;
esac

# Check for required tools
if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl -sSL"
    API_DOWNLOADER="curl -sS"
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget -qO-"
    API_DOWNLOADER="wget -qO-"
else
    error "Neither curl nor wget is installed. Please install one of them and try again."
fi

# Fetch the latest release version
info "Fetching latest release version..."
LATEST_RELEASE=$(${API_DOWNLOADER} "${GITHUB_API_URL}")

# Parse the tag_name from JSON response
TAG=$(echo "${LATEST_RELEASE}" | grep -o '"tag_name": *"[^"]*"' | sed 's/"tag_name": "//' | sed 's/"//')

if [ -z "${TAG}" ]; then
    error "Failed to retrieve the latest release tag."
fi

info "Latest version is ${TAG}"

# Construct download URL
FILENAME="iqos_cli-${TAG}-${ASSET_PACKAGE}.tar.gz"
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG}/${FILENAME}"

# Determine installation directory
if [ -d "$HOME/.local/bin" ] && [ -w "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    SUDO=""
elif mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    INSTALL_DIR="$HOME/.local/bin"
    SUDO=""
else
    INSTALL_DIR="/usr/local/bin"
    SUDO="sudo"
fi

info "Downloading ${FILENAME}..."
TMP_DIR=$(mktemp -d)
# Ensure cleanup on exit
trap 'rm -rf "${TMP_DIR}"' EXIT

${DOWNLOADER} "${DOWNLOAD_URL}" > "${TMP_DIR}/${FILENAME}"

info "Extracting..."
tar -xzf "${TMP_DIR}/${FILENAME}" -C "${TMP_DIR}"

EXTRACTED_DIR="${TMP_DIR}/iqos_cli-${TAG}-${ASSET_PACKAGE}"
BINARY="${EXTRACTED_DIR}/iqos"

if [ ! -f "${BINARY}" ]; then
    error "Binary not found in the downloaded archive."
fi

info "Installing to ${INSTALL_DIR}..."
if [ -n "${SUDO}" ]; then
    info "Sudo privileges may be required to install to ${INSTALL_DIR}"
fi

${SUDO} mkdir -p "${INSTALL_DIR}"
${SUDO} mv "${BINARY}" "${INSTALL_DIR}/iqos"
${SUDO} chmod +x "${INSTALL_DIR}/iqos"

info "Installation complete! You can now run 'iqos' from your terminal."

if ! echo "$PATH" | grep -q "${INSTALL_DIR}"; then
    echo "================================================================"
    echo " NOTE: ${INSTALL_DIR} is not in your PATH."
    echo " Please add it to your shell configuration (e.g. ~/.bashrc or ~/.zshrc)."
    echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo "================================================================"
fi

#!/bin/bash
#
# install-native.sh - Download and install native libraries for gopher-mcp-rust
#
# Usage:
#   ./install-native.sh [VERSION] [INSTALL_DIR]
#
# Arguments:
#   VERSION     - Version to install (default: latest)
#   INSTALL_DIR - Installation directory (default: ./native)
#
# Examples:
#   ./install-native.sh                    # Install latest to ./native
#   ./install-native.sh v0.1.2             # Install specific version
#   ./install-native.sh latest /usr/local  # Install to /usr/local
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

VERSION="${1:-latest}"
INSTALL_DIR="${2:-./native}"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  gopher-mcp-rust Native Library Installer${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Check for gh CLI
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install it with: brew install gh"
    echo "Then authenticate: gh auth login"
    exit 1
fi

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin) OS_NAME="macos" ;;
    linux) OS_NAME="linux" ;;
    mingw*|msys*|cygwin*) OS_NAME="windows" ;;
    *) echo -e "${RED}Error: Unsupported OS: $OS${NC}"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH_NAME="x64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *) echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

PLATFORM="${OS_NAME}-${ARCH_NAME}"
echo -e "Detected platform: ${GREEN}${PLATFORM}${NC}"

# Determine file extension
if [ "$OS_NAME" = "windows" ]; then
    ARCHIVE_EXT="zip"
else
    ARCHIVE_EXT="tar.gz"
fi

ARCHIVE_NAME="libgopher-orch-${PLATFORM}.${ARCHIVE_EXT}"

# Get version if "latest"
if [ "$VERSION" = "latest" ]; then
    echo -e "${YELLOW}Fetching latest version...${NC}"
    VERSION=$(gh release view -R GopherSecurity/gopher-mcp-rust --json tagName -q '.tagName' 2>/dev/null) || {
        echo -e "${RED}Error: Could not fetch latest release${NC}"
        echo "Make sure the repository has releases and you have access."
        exit 1
    }
fi

echo -e "Version: ${GREEN}${VERSION}${NC}"
echo -e "Archive: ${GREEN}${ARCHIVE_NAME}${NC}"
echo -e "Install directory: ${GREEN}${INSTALL_DIR}${NC}"
echo ""

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

cd "$TEMP_DIR"

# Download
echo -e "${YELLOW}Downloading native library...${NC}"
gh release download "$VERSION" \
    -R GopherSecurity/gopher-mcp-rust \
    -p "$ARCHIVE_NAME" || {
    echo -e "${RED}Error: Could not download $ARCHIVE_NAME${NC}"
    echo ""
    echo "Available assets for $VERSION:"
    gh release view "$VERSION" -R GopherSecurity/gopher-mcp-rust --json assets -q '.assets[].name'
    exit 1
}

echo -e "${GREEN}✓ Downloaded${NC}"

# Extract
echo -e "${YELLOW}Extracting...${NC}"

if [ "$ARCHIVE_EXT" = "zip" ]; then
    unzip -o "$ARCHIVE_NAME"
else
    tar -xzf "$ARCHIVE_NAME"
fi

echo -e "${GREEN}✓ Extracted${NC}"

# Get absolute path for install directory
ORIGINAL_DIR=$(pwd)
cd - > /dev/null
INSTALL_DIR=$(cd "$(dirname "$INSTALL_DIR")" 2>/dev/null && pwd)/$(basename "$INSTALL_DIR") || INSTALL_DIR="$PWD/$INSTALL_DIR"
cd "$TEMP_DIR"

# Install
echo -e "${YELLOW}Installing to ${INSTALL_DIR}...${NC}"

# Check if we need sudo
NEED_SUDO=""
if [ ! -w "$(dirname "$INSTALL_DIR")" ] 2>/dev/null && [ ! -d "$INSTALL_DIR" ]; then
    if [ ! -w "$INSTALL_DIR" ] 2>/dev/null; then
        NEED_SUDO="sudo"
        echo -e "${YELLOW}  (requires sudo)${NC}"
    fi
fi

# Create directories
$NEED_SUDO mkdir -p "${INSTALL_DIR}/lib"
$NEED_SUDO mkdir -p "${INSTALL_DIR}/include"

# Copy libraries
if [ -d "lib" ]; then
    $NEED_SUDO cp -P lib/* "${INSTALL_DIR}/lib/" 2>/dev/null || true
fi

# Copy headers
if [ -d "include" ]; then
    $NEED_SUDO cp -r include/* "${INSTALL_DIR}/include/" 2>/dev/null || true
fi

# Handle flat structure (files directly in archive)
$NEED_SUDO cp -P *.dylib "${INSTALL_DIR}/lib/" 2>/dev/null || true
$NEED_SUDO cp -P *.so* "${INSTALL_DIR}/lib/" 2>/dev/null || true
$NEED_SUDO cp -P *.dll "${INSTALL_DIR}/lib/" 2>/dev/null || true
$NEED_SUDO cp -P *.h "${INSTALL_DIR}/include/" 2>/dev/null || true

echo -e "${GREEN}✓ Installed${NC}"
echo ""

# Show installed files
echo -e "${CYAN}Installed files:${NC}"
echo "  Libraries:"
ls -la "${INSTALL_DIR}/lib/"*gopher* 2>/dev/null | sed 's/^/    /' || echo "    (none found)"
echo "  Headers:"
ls -la "${INSTALL_DIR}/include/"*gopher* 2>/dev/null | sed 's/^/    /' || \
ls -la "${INSTALL_DIR}/include/"orch* 2>/dev/null | sed 's/^/    /' || echo "    (none found)"
echo ""

# Print environment setup
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}For Rust projects:${NC}"
echo ""
echo "  # Add to Cargo.toml"
echo "  [dependencies]"
echo "  gopher-orch = \"${VERSION#v}\""
echo ""
echo -e "${YELLOW}Set environment variables:${NC}"
echo ""

if [ "$OS_NAME" = "macos" ]; then
    echo "  export DYLD_LIBRARY_PATH=\"${INSTALL_DIR}/lib:\$DYLD_LIBRARY_PATH\""
elif [ "$OS_NAME" = "linux" ]; then
    echo "  export LD_LIBRARY_PATH=\"${INSTALL_DIR}/lib:\$LD_LIBRARY_PATH\""
    echo ""
    echo -e "${YELLOW}Or add to system library path:${NC}"
    echo "  echo '${INSTALL_DIR}/lib' | sudo tee /etc/ld.so.conf.d/gopher-orch.conf"
    echo "  sudo ldconfig"
fi

echo ""
echo -e "${CYAN}To verify installation:${NC}"
echo "  cargo build"
echo ""

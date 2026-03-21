#!/bin/bash
#
# Gopher Auth MCP Server - Run Script (Rust)
# This script downloads dependencies and runs the Rust auth example server
# Works as a standalone third-party example
#
# Usage:
#   ./run_example.sh                    # Run with default config
#   ./run_example.sh --no-auth          # Run with auth disabled
#   ./run_example.sh --config <file>    # Run with custom config file
#   ./run_example.sh --skip-download    # Skip native library download
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SDK_VERSION="${SDK_VERSION:-v0.1.2}"
NATIVE_LIB_DIR="${NATIVE_LIB_DIR:-$SCRIPT_DIR/native/lib}"
NATIVE_INCLUDE_DIR="${NATIVE_INCLUDE_DIR:-$SCRIPT_DIR/native/include}"
GITHUB_REPO="GopherSecurity/gopher-mcp-rust"

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --no-auth         Run with authentication disabled"
    echo "  --config <file>   Use custom configuration file"
    echo "  --skip-download   Skip native library download (use existing)"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  SDK_VERSION       Version of gopher-mcp-rust SDK (default: $SDK_VERSION)"
    echo "  NATIVE_LIB_DIR    Directory for native libraries (default: ./native/lib)"
    echo ""
    echo "Examples:"
    echo "  $0                         # Run with default settings"
    echo "  $0 --no-auth               # Run with auth disabled"
    echo "  SDK_VERSION=v0.1.3 $0      # Use specific SDK version"
    echo ""
}

# Check for cargo
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: cargo is not installed${NC}"
        echo "Please install Rust from https://rustup.rs/"
        exit 1
    fi

    RUST_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
    echo -e "${GREEN}Rust version: $RUST_VERSION${NC}"
}

# Check for gh CLI
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
        echo "Install it with: brew install gh"
        echo "Then authenticate: gh auth login"
        exit 1
    fi
}

# Download native library
download_native_library() {
    echo -e "${YELLOW}Downloading native library ($SDK_VERSION)...${NC}"

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

    # Determine file extension
    if [ "$OS_NAME" = "windows" ]; then
        ARCHIVE_EXT="zip"
    else
        ARCHIVE_EXT="tar.gz"
    fi

    ARCHIVE_NAME="libgopher-orch-${PLATFORM}.${ARCHIVE_EXT}"

    echo -e "  Platform: ${GREEN}${PLATFORM}${NC}"
    echo -e "  Archive: ${GREEN}${ARCHIVE_NAME}${NC}"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    cd "$TEMP_DIR"

    # Download
    gh release download "$SDK_VERSION" \
        -R "$GITHUB_REPO" \
        -p "$ARCHIVE_NAME" || {
        echo -e "${RED}Error: Could not download $ARCHIVE_NAME${NC}"
        echo ""
        echo "Available assets for $SDK_VERSION:"
        gh release view "$SDK_VERSION" -R "$GITHUB_REPO" --json assets -q '.assets[].name' 2>/dev/null || echo "  (could not list assets)"
        exit 1
    }

    echo -e "${GREEN}Downloaded${NC}"

    # Extract
    echo -e "${YELLOW}Extracting...${NC}"

    if [ "$ARCHIVE_EXT" = "zip" ]; then
        unzip -o "$ARCHIVE_NAME"
    else
        tar -xzf "$ARCHIVE_NAME"
    fi

    # Create directories
    mkdir -p "$NATIVE_LIB_DIR"
    mkdir -p "$NATIVE_INCLUDE_DIR"

    # Copy libraries
    if [ -d "lib" ]; then
        cp -P lib/* "$NATIVE_LIB_DIR/" 2>/dev/null || true
    fi

    # Copy headers
    if [ -d "include" ]; then
        cp -r include/* "$NATIVE_INCLUDE_DIR/" 2>/dev/null || true
    fi

    # Handle flat structure (files directly in archive)
    cp -P *.dylib "$NATIVE_LIB_DIR/" 2>/dev/null || true
    cp -P *.so* "$NATIVE_LIB_DIR/" 2>/dev/null || true
    cp -P *.dll "$NATIVE_LIB_DIR/" 2>/dev/null || true
    cp -P *.h "$NATIVE_INCLUDE_DIR/" 2>/dev/null || true

    cd "$SCRIPT_DIR"

    echo -e "${GREEN}Native library installed to $NATIVE_LIB_DIR${NC}"
}

# Check if native library exists
check_native_library() {
    if [ -f "$NATIVE_LIB_DIR/libgopher-orch.dylib" ] || [ -f "$NATIVE_LIB_DIR/libgopher-orch.so" ] || \
       [ -f "$NATIVE_LIB_DIR/libgopher-orch.0.dylib" ] || [ -f "$NATIVE_LIB_DIR/libgopher-orch.0.so" ] || \
       ls "$NATIVE_LIB_DIR"/libgopher-orch*.dylib 1> /dev/null 2>&1 || \
       ls "$NATIVE_LIB_DIR"/libgopher-orch*.so 1> /dev/null 2>&1; then
        echo -e "${GREEN}Native library found at $NATIVE_LIB_DIR${NC}"
        return 0
    fi
    return 1
}

# Parse arguments
CONFIG_FILE="server.config"
AUTH_DISABLED=false
SKIP_DOWNLOAD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-auth)
            AUTH_DISABLED=true
            shift
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --skip-download)
            SKIP_DOWNLOAD=true
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

echo "========================================="
echo "  Gopher Auth MCP Server (Rust)"
echo "========================================="
echo ""

# Check Rust/Cargo
check_cargo

# Download native library if needed
if [ "$SKIP_DOWNLOAD" = false ]; then
    if check_native_library; then
        echo -e "${YELLOW}Using existing native library. Use --skip-download=false to re-download.${NC}"
    else
        check_gh_cli
        download_native_library
    fi
else
    if ! check_native_library; then
        echo -e "${RED}Error: Native library not found and --skip-download specified${NC}"
        exit 1
    fi
fi

echo ""

# Set environment for native library loading
export DYLD_LIBRARY_PATH="${NATIVE_LIB_DIR}:${DYLD_LIBRARY_PATH}"
export LD_LIBRARY_PATH="${NATIVE_LIB_DIR}:${LD_LIBRARY_PATH}"
export LIBRARY_PATH="${NATIVE_LIB_DIR}:${LIBRARY_PATH}"

# Set log level
export RUST_LOG="${RUST_LOG:-info}"

# Build the project
echo "Building auth-mcp-server..."
cargo build --release
echo -e "${GREEN}Build successful${NC}"
echo ""

# Create temporary config if --no-auth was specified
if [ "$AUTH_DISABLED" = true ]; then
    echo "Running with authentication disabled..."
    TEMP_CONFIG=$(mktemp)
    cat > "$TEMP_CONFIG" << EOF
# Temporary config with auth disabled
host=0.0.0.0
port=3001
server_url=http://localhost:3001
auth_disabled=true
allowed_scopes=openid profile email mcp:read mcp:admin
EOF
    CONFIG_FILE="$TEMP_CONFIG"
    trap "rm -f $TEMP_CONFIG" EXIT
fi

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Warning: Config file '$CONFIG_FILE' not found${NC}"
    echo "Server will use default configuration with auth disabled"
fi

# Run the server
echo "Starting Rust Auth MCP Server..."
echo ""
./target/release/auth-mcp-server "$CONFIG_FILE"

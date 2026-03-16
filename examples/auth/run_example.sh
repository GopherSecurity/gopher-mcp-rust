#!/bin/bash
#
# Run the Rust Auth MCP Server example
#
# Usage:
#   ./run_example.sh                    # Run with default config
#   ./run_example.sh --no-auth          # Run with auth disabled
#   ./run_example.sh --config <file>    # Run with custom config file
#

set -e

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Parse arguments
CONFIG_FILE="server.config"
AUTH_DISABLED=false

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
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-auth         Run with authentication disabled"
            echo "  --config <file>   Use custom configuration file"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build the project
echo "Building auth-mcp-server..."
cargo build --release

# Set log level
export RUST_LOG="${RUST_LOG:-info}"

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
    echo "Warning: Config file '$CONFIG_FILE' not found"
    echo "Server will use default configuration with auth disabled"
fi

# Run the server
echo "Starting Rust Auth MCP Server..."
echo ""
./target/release/auth-mcp-server "$CONFIG_FILE"

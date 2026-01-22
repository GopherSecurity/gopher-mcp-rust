#!/bin/bash

# Run the Rust client example with local MCP servers

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    # Kill the process groups to ensure child processes (tsx/node) are also killed
    if [ -n "$SERVER3001_PID" ]; then
        kill -- -$SERVER3001_PID 2>/dev/null || kill $SERVER3001_PID 2>/dev/null || true
    fi
    if [ -n "$SERVER3002_PID" ]; then
        kill -- -$SERVER3002_PID 2>/dev/null || kill $SERVER3002_PID 2>/dev/null || true
    fi
    # Also kill any remaining tsx/node processes on our ports
    lsof -ti:3001 | xargs kill 2>/dev/null || true
    lsof -ti:3002 | xargs kill 2>/dev/null || true
    echo -e "${GREEN}Done${NC}"
}

trap cleanup EXIT

echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Running Rust Client Example${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""

# Check if native library exists
if [ ! -d "$PROJECT_DIR/native/lib" ]; then
    echo -e "${RED}Error: Native library not found at $PROJECT_DIR/native/lib${NC}"
    echo -e "${YELLOW}Please run ./build.sh first${NC}"
    exit 1
fi

# Start server3001
echo -e "${YELLOW}Starting server3001...${NC}"
cd "$SCRIPT_DIR/server3001"
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies for server3001...${NC}"
    npm install
fi
npm run dev &
SERVER3001_PID=$!
echo -e "${GREEN}server3001 started (PID: $SERVER3001_PID)${NC}"

# Start server3002
echo -e "${YELLOW}Starting server3002...${NC}"
cd "$SCRIPT_DIR/server3002"
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies for server3002...${NC}"
    npm install
fi
npm run dev &
SERVER3002_PID=$!
echo -e "${GREEN}server3002 started (PID: $SERVER3002_PID)${NC}"

# Wait for servers to start
echo -e "${YELLOW}Waiting for servers to start...${NC}"
sleep 3

# Build and run the Rust example
echo ""
echo -e "${YELLOW}Building and running Rust client...${NC}"
echo ""
cd "$PROJECT_DIR"

# Build the example
LIBRARY_PATH="$PROJECT_DIR/native/lib" \
LD_LIBRARY_PATH="$PROJECT_DIR/native/lib" \
DYLD_LIBRARY_PATH="$PROJECT_DIR/native/lib" \
cargo build --example client_example_json --release

# Run the example
LIBRARY_PATH="$PROJECT_DIR/native/lib" \
LD_LIBRARY_PATH="$PROJECT_DIR/native/lib" \
DYLD_LIBRARY_PATH="$PROJECT_DIR/native/lib" \
./target/release/examples/client_example_json "$@"

echo ""
echo -e "${GREEN}Example completed${NC}"

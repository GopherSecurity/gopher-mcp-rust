# gopher-mcp-rust - Rust SDK

Rust SDK for Gopher Orch - AI Agent orchestration framework with native C++ performance.

## Table of Contents

- [Features](#features)
- [When to Use This SDK](#when-to-use-this-sdk)
- [Architecture](#architecture)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Building from Source](#building-from-source)
  - [Prerequisites](#prerequisites)
  - [Step 1: Clone the Repository](#step-1-clone-the-repository)
  - [Step 2: Build Everything](#step-2-build-everything)
  - [Step 3: Verify the Build](#step-3-verify-the-build)
  - [Step 4: Run Tests](#step-4-run-tests)
- [Native Library Details](#native-library-details)
  - [Library Location](#library-location)
  - [Platform-Specific Library Names](#platform-specific-library-names)
  - [Library Search Order](#library-search-order)
- [API Documentation](#api-documentation)
  - [GopherAgent](#gopheragent)
  - [ConfigBuilder](#configbuilder)
  - [Error Handling](#error-handling)
- [Examples](#examples)
  - [Basic Usage with API Key](#basic-usage-with-api-key)
  - [Using Local MCP Servers](#using-local-mcp-servers)
  - [Running the Example](#running-the-example)
- [Development](#development)
  - [Project Structure](#project-structure)
  - [Build Scripts](#build-scripts)
  - [Rebuilding Native Library](#rebuilding-native-library)
  - [Updating Submodules](#updating-submodules)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)
- [Links](#links)
- [Acknowledgments](#acknowledgments)

---

## Features

- **Native Performance** - Powered by C++ core with Rust bindings via libloading
- **AI Agent Framework** - Build intelligent agents with LLM integration
- **MCP Protocol** - Model Context Protocol client and server support
- **Tool Orchestration** - Manage and execute tools across multiple MCP servers
- **State Management** - Built-in state graph for complex workflows
- **Memory Safety** - Rust's ownership system with zero-cost abstractions

## When to Use This SDK

This SDK is ideal for:

- **Rust applications** that need high-performance AI agent orchestration
- **Systems programming** requiring MCP protocol support with memory safety
- **CLI tools** integrating AI agents with native performance
- **WebAssembly targets** (with appropriate native library builds)
- **Embedded systems** needing lightweight agent infrastructure

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Application                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rust SDK (gopher_mcp_rust)                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ GopherAgent │  │ConfigBuilder│  │ Error Types         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │ FFI (libloading)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Native Library (libgopher-orch)                │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐  │
│  │ Agent Engine  │  │ LLM Providers │  │ MCP Client      │  │
│  │               │  │ - Anthropic   │  │ - HTTP/SSE      │  │
│  │               │  │ - OpenAI      │  │ - Tool Registry │  │
│  └───────────────┘  └───────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    MCP Servers                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Weather API │  │ Database    │  │ Custom Tools        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Installation

### Option 1: Cargo (when published)

```toml
[dependencies]
gopher-mcp-rust ="0.1.0"
```

### Option 2: Git Dependency

```toml
[dependencies]
gopher-mcp-rust ={ git = "https://github.com/GopherSecurity/gopher-mcp-rust.git" }
```

### Option 3: Build from Source

See [Building from Source](#building-from-source) section below.

## Quick Start

```rust
use gopher_mcp_rust::{GopherAgent, ConfigBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent with API key (fetches server config from remote API)
    let config = ConfigBuilder::new()
        .with_provider("AnthropicProvider")
        .with_model("claude-3-haiku-20240307")
        .with_api_key("your-api-key")
        .build();

    let agent = GopherAgent::create(config)?;

    // Run the agent
    let result = agent.run("What is the weather in Tokyo?")?;
    println!("{}", result);

    // Agent is automatically cleaned up when dropped
    Ok(())
}
```

---

## Building from Source

This SDK wraps a native C++ library via libloading. You must build the native library before using the SDK.

### Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Rust | >= 1.64 | Stable toolchain |
| Cargo | Latest | Comes with Rust |
| Git | Latest | For cloning and submodules |
| CMake | >= 3.15 | Native library build system |
| C++ Compiler | C++14+ | Clang (macOS), GCC (Linux), MSVC (Windows) |

**Platform-specific requirements:**

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: `build-essential`, `libssl-dev`
- **Windows**: Visual Studio 2019+ with C++ workload

### Step 1: Clone the Repository

```bash
git clone https://github.com/GopherSecurity/gopher-mcp-rust.git
cd gopher-mcp-rust
```

### Step 2: Build Everything

**Using build.sh (recommended)**

The `build.sh` script handles everything automatically:

```bash
./build.sh
```

**Using build.sh with Multiple GitHub Accounts:**

If you have multiple GitHub accounts configured with SSH host aliases, use the `GITHUB_SSH_HOST` environment variable:

```bash
# Use custom SSH host alias for cloning private submodules
GITHUB_SSH_HOST=your-ssh-alias ./build.sh

# Example: if your ~/.ssh/config has "Host github-work" for work account
GITHUB_SSH_HOST=github-work ./build.sh
```

**What happens during build:**

1. **Submodule update** - Initializes and updates submodules (with SSH URL rewriting if `GITHUB_SSH_HOST` is set)
2. **CMake configure** - Configures the C++ build with Release settings
3. **Native compilation** - Compiles C++ to shared libraries
4. **Library installation** - Copies libraries to `native/lib/`
5. **Dependency copying** - Copies required dependencies (gopher-mcp, fmt)
6. **macOS fixes** - Fixes dylib install names for proper runtime loading
7. **Cargo build** - Compiles Rust SDK
8. **Tests** - Runs Cargo tests

### Step 3: Verify the Build

```bash
# Check native libraries were built
ls -la native/lib/

# Expected output (macOS):
# libgopher-orch.dylib
# libgopher-mcp.dylib
# libgopher-mcp-event.dylib
# libfmt.dylib

# Verify Rust build
cargo build
```

### Step 4: Run Tests

```bash
cargo test
```

---

## Native Library Details

### Library Location

After building, native libraries are installed to:

```
native/
├── lib/                          # Shared libraries
│   ├── libgopher-orch.dylib     # Main orchestration library (macOS)
│   ├── libgopher-orch.so        # Main orchestration library (Linux)
│   ├── libgopher-mcp.dylib      # MCP protocol library
│   ├── libgopher-mcp-event.dylib # Event handling
│   └── libfmt.dylib             # Formatting library
└── include/                      # C++ headers (for development)
    └── orch/
        └── core/
```

### Platform-Specific Library Names

| Platform | Library Extension | Example |
|----------|------------------|---------|
| macOS | `.dylib` | `libgopher-orch.dylib` |
| Linux | `.so` | `libgopher-orch.so` |
| Windows | `.dll` | `gopher-orch.dll` |

### Library Search Order

The SDK searches for the native library in this order:

1. `native/lib/` relative to executable
2. `./native/lib/` relative to working directory
3. `CARGO_MANIFEST_DIR/native/lib/` (compile time)
4. System library paths

---

## API Documentation

### GopherAgent

The main struct for creating and running AI agents:

```rust
use gopher_mcp_rust::{GopherAgent, ConfigBuilder, AgentResult};

// Initialize the library (called automatically on first create)
gopher_mcp_rust::init()?;

// Create with API key (fetches server config from remote API)
let config = ConfigBuilder::new()
    .with_provider("AnthropicProvider")
    .with_model("claude-3-haiku-20240307")
    .with_api_key("your-api-key")
    .build();

let agent = GopherAgent::create(config)?;

// Or create with JSON server config
let server_config = r#"
    {
        "succeeded": true,
        "data": {
            "servers": [{
                "serverId": "server1",
                "name": "My MCP Server",
                "transport": "http_sse",
                "config": {"url": "http://localhost:3001/mcp"}
            }]
        }
    }
"#;

let agent = GopherAgent::create_with_server_config(
    "AnthropicProvider",
    "claude-3-haiku-20240307",
    server_config,
)?;

// Run a query
let result = agent.run("Your prompt here")?;

// Run with custom timeout (default: 60000ms)
let result = agent.run_with_timeout("Your prompt here", 30000)?;

// Run with detailed result information
let detailed: AgentResult = agent.run_detailed("Your prompt here");
// Returns AgentResult with: response(), status(), iteration_count(), tokens_used()

// Manual cleanup (optional - happens automatically on drop)
drop(agent);

// Shutdown library
gopher_mcp_rust::shutdown();
```

### ConfigBuilder

Builder for creating agent configurations:

```rust
use gopher_mcp_rust::ConfigBuilder;

// With API key
let config = ConfigBuilder::new()
    .with_provider("AnthropicProvider")
    .with_model("claude-3-haiku-20240307")
    .with_api_key("your-api-key")
    .build();

// With server config
let config = ConfigBuilder::new()
    .with_provider("AnthropicProvider")
    .with_model("claude-3-haiku-20240307")
    .with_server_config(r#"{"succeeded": true, "data": {"servers": []}}"#)
    .build();

// Check configuration
assert!(config.has_api_key());
assert!(!config.has_server_config());
```

### Error Handling

The SDK provides typed errors for different failure scenarios:

```rust
use gopher_mcp_rust::{GopherAgent, ConfigBuilder, Error};

fn main() {
    let config = ConfigBuilder::new()
        .with_provider("AnthropicProvider")
        .with_model("claude-3-haiku-20240307")
        .with_api_key("invalid-key")
        .build();

    match GopherAgent::create(config) {
        Ok(agent) => {
            match agent.run("query") {
                Ok(result) => println!("{}", result),
                Err(Error::Timeout(msg)) => eprintln!("Query timed out: {}", msg),
                Err(e) => eprintln!("Query error: {}", e),
            }
        }
        Err(Error::Library(msg)) => eprintln!("Library not found: {}", msg),
        Err(Error::Config(msg)) => eprintln!("Invalid config: {}", msg),
        Err(Error::Agent(msg)) => eprintln!("Agent error: {}", msg),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## Examples

### Basic Usage with API Key

```rust
use gopher_mcp_rust::{GopherAgent, ConfigBuilder};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GOPHER_API_KEY")?;

    let config = ConfigBuilder::new()
        .with_provider("AnthropicProvider")
        .with_model("claude-3-haiku-20240307")
        .with_api_key(&api_key)
        .build();

    let agent = GopherAgent::create(config)?;

    let answer = agent.run("What time is it in London?")?;
    println!("Answer: {}", answer);

    Ok(())
}
```

### Using Local MCP Servers

```rust
use gopher_mcp_rust::{GopherAgent, ConfigBuilder};

const SERVER_CONFIG: &str = r#"{
    "succeeded": true,
    "code": 200,
    "message": "OK",
    "data": {
        "servers": [
            {
                "version": "1.0.0",
                "serverId": "weather-server",
                "name": "Weather Service",
                "transport": "http_sse",
                "config": {
                    "url": "http://localhost:3001/mcp",
                    "headers": {}
                },
                "connectTimeout": 5000,
                "requestTimeout": 30000
            }
        ]
    }
}"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .with_provider("AnthropicProvider")
        .with_model("claude-3-haiku-20240307")
        .with_server_config(SERVER_CONFIG)
        .build();

    let agent = GopherAgent::create(config)?;

    let result = agent.run("What is the weather in New York?")?;
    println!("{}", result);

    Ok(())
}
```

### Running the Example

```bash
# Run with the convenience script (starts servers automatically)
cd examples
./client_example_json_run.sh

# Or manually:
# Terminal 1: Start server3001
cd examples/server3001 && npm install && npm run dev

# Terminal 2: Start server3002
cd examples/server3002 && npm install && npm run dev

# Terminal 3: Run the Rust client
ANTHROPIC_API_KEY=your-key cargo run --example client_example_json
```

---

## Development

### Project Structure

```
gopher-mcp-rust/
├── src/
│   ├── lib.rs                    # Library entry point
│   ├── agent.rs                  # GopherAgent implementation
│   ├── config.rs                 # Configuration builder
│   ├── error.rs                  # Error types
│   ├── result.rs                 # AgentResult types
│   └── ffi.rs                    # FFI bindings (libloading)
├── tests/
│   └── ffi_tests.rs              # Integration tests
├── native/                       # Native libraries (generated)
│   ├── lib/                      # Shared libraries (.dylib, .so, .dll)
│   └── include/                  # C++ headers
├── third_party/                  # Git submodules
│   └── gopher-orch/              # C++ implementation
├── examples/                     # Example code
│   ├── client_example_json.rs
│   ├── client_example_json_run.sh
│   ├── server3001/               # Mock weather MCP server
│   └── server3002/               # Mock tools MCP server
├── build.sh                      # Build orchestration script
├── Cargo.toml                    # Cargo build configuration
└── README.md
```

### Build Scripts

| Script | Description |
|--------|-------------|
| `./build.sh` | Full build (submodules + native + Rust SDK) |
| `GITHUB_SSH_HOST=alias ./build.sh` | Build with custom SSH host |
| `cargo build` | Compile Rust SDK |
| `cargo test` | Run tests |
| `cargo build --release` | Release build |
| `cargo run --example client_example_json` | Run example |

### Rebuilding Native Library

If you modify the C++ code or switch branches:

```bash
# Clean and rebuild
rm -rf third_party/gopher-orch/build native/lib
./build.sh
```

### Updating Submodules

To pull latest changes from native libraries:

```bash
# Update to latest commit
cd third_party/gopher-orch
git fetch origin
git checkout <commit-or-branch>
cd ../..

# Rebuild
rm -rf native/lib
./build.sh
```

---

## Troubleshooting

### "Library not found" Error

**Cause**: Native library not built or not in expected location.

**Solution**:
```bash
# Rebuild native library
./build.sh

# Verify library exists
ls native/lib/libgopher-orch.*
```

### "Submodule is empty" Error

**Cause**: Git submodules not initialized.

**Solution**:
```bash
git submodule update --init --recursive
```

### CMake Configuration Fails

**Cause**: Missing dependencies or wrong CMake version.

**Solution**:
```bash
# macOS
brew install cmake

# Linux (Ubuntu/Debian)
sudo apt-get install cmake build-essential libssl-dev

# Verify version
cmake --version  # Should be >= 3.15
```

### Linking Error at Runtime

**Cause**: libloading can't find the native library.

**Solution**:
```bash
# Run from project root
cargo run --example client_example_json

# Or set library path
export DYLD_LIBRARY_PATH=$PWD/native/lib:$DYLD_LIBRARY_PATH  # macOS
export LD_LIBRARY_PATH=$PWD/native/lib:$LD_LIBRARY_PATH      # Linux
```

### Build Fails on Apple Silicon (M1/M2)

**Cause**: Architecture mismatch.

**Solution**:
```bash
# Ensure using native arm64 toolchain
arch -arm64 ./build.sh
```

### Rust Version Compatibility

**Cause**: Using older Rust toolchain.

**Solution**:
```bash
# Update Rust
rustup update stable

# Or use minimum supported version
rustup install 1.64
rustup default 1.64
```

---

## Contributing

Contributions are welcome! Please read our contributing guidelines.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Ensure submodules are initialized (`git submodule update --init --recursive`)
4. Make your changes
5. Run tests (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Links

- [GitHub Repository](https://github.com/GopherSecurity/gopher-mcp-rust)
- [Java SDK](https://github.com/GopherSecurity/gopher-mcp-java)
- [Python SDK](https://github.com/GopherSecurity/gopher-mcp-python)
- [TypeScript SDK](https://github.com/GopherSecurity/gopher-orch-js)
- [Native C++ Implementation](https://github.com/GopherSecurity/gopher-orch)
- [Model Context Protocol](https://modelcontextprotocol.io/)

## Acknowledgments

- Built on [gopher-orch](https://github.com/GopherSecurity/gopher-orch) C++ framework
- Uses [gopher-mcp](https://github.com/GopherSecurity/gopher-mcp) for MCP protocol
- Inspired by LangChain and LangGraph
- FFI bindings via [libloading](https://github.com/nagisa/rust_libloading)

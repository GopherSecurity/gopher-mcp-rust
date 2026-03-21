# Gopher Auth MCP Server - Rust Example

This example demonstrates an MCP (Model Context Protocol) server with OAuth 2.0 authentication using the gopher-mcp-rust SDK.

## Overview

The auth example server provides:
- OAuth 2.0 / OIDC discovery endpoints (RFC 8414, RFC 9728)
- JWT token validation via native library
- Scope-based authorization for MCP tools
- Example weather tools with different scope requirements

## Prerequisites

- Rust 1.70 or later
- GitHub CLI (`gh`) for downloading native libraries

## Installation

### 1. Clone or Copy This Example

```bash
# Option A: Clone the repository
git clone https://github.com/GopherSecurity/gopher-mcp-rust.git
cd gopher-mcp-rust/examples/auth

# Option B: Copy the example files to your project
# Copy the examples/auth directory contents
```

### 2. Install the Rust SDK

The SDK is specified in `Cargo.toml` as a git dependency:

```toml
[dependencies]
gopher-orch = { git = "https://github.com/GopherSecurity/gopher-mcp-rust.git", features = ["auth"] }
```

### 3. Download Native Libraries

The SDK requires native libraries for OAuth token validation. The `run_example.sh` script downloads these automatically, or you can install them manually:

```bash
# Using the run script (downloads automatically)
./run_example.sh --no-auth

# Or download manually using the install script
curl -sSL https://raw.githubusercontent.com/GopherSecurity/gopher-mcp-rust/main/install-native.sh | bash -s -- latest ./native
```

## Quick Start

### Development Mode (No Auth)

```bash
# Run with auth disabled (all requests bypass authentication)
./run_example.sh --no-auth

# Or build and run manually
cargo build --release
./target/release/auth-mcp-server
```

### With Full OAuth Support

```bash
# Run with OAuth authentication enabled
./run_example.sh

# Or build manually with environment set
export DYLD_LIBRARY_PATH="./native/lib:$DYLD_LIBRARY_PATH"
cargo build --release
./target/release/auth-mcp-server server.config
```

### Using Environment Variables

```bash
# Use a specific SDK version
SDK_VERSION=v0.1.3 ./run_example.sh

# Use custom native library location
NATIVE_LIB_DIR=/usr/local/lib ./run_example.sh --skip-download
```

## Configuration

Create a `server.config` file with INI-style key-value pairs:

```ini
# Server settings
host=0.0.0.0
port=3001
server_url=http://localhost:3001

# OAuth/IDP settings
client_id=my-client
client_secret=my-secret
auth_server_url=https://keycloak.example.com/realms/mcp

# Scopes
allowed_scopes=openid profile email mcp:read mcp:admin

# Cache settings
jwks_cache_duration=3600
jwks_auto_refresh=true
request_timeout=30

# Auth bypass mode (for development)
auth_disabled=true
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `host` | Bind address | `0.0.0.0` |
| `port` | Port number | `3001` |
| `server_url` | Public URL of this server | `http://localhost:3001` |
| `auth_server_url` | Keycloak/IDP base URL | - |
| `client_id` | OAuth client ID | - |
| `client_secret` | OAuth client secret | - |
| `allowed_scopes` | Space-separated allowed scopes | - |
| `jwks_cache_duration` | JWKS cache TTL in seconds | `3600` |
| `jwks_auto_refresh` | Auto-refresh JWKS | `true` |
| `request_timeout` | HTTP request timeout in seconds | `30` |
| `auth_disabled` | Disable authentication | `false` |

## Available Endpoints

### Health Check

```bash
curl http://localhost:3001/health
```

### OAuth Discovery

```bash
# Protected Resource Metadata (RFC 9728)
curl http://localhost:3001/.well-known/oauth-protected-resource

# Authorization Server Metadata (RFC 8414)
curl http://localhost:3001/.well-known/oauth-authorization-server

# OpenID Connect Discovery
curl http://localhost:3001/.well-known/openid-configuration
```

### MCP Endpoints

```bash
# Initialize
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}'

# List Tools
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

# Call Tool (with auth)
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get-weather","arguments":{"city":"NYC"}}}'
```

## Available Tools

| Tool | Scope Required | Description |
|------|----------------|-------------|
| `get-weather` | None | Get current weather for a city |
| `get-forecast` | `mcp:read` | Get 5-day weather forecast |
| `get-weather-alerts` | `mcp:admin` | Get weather alerts for a region |

### Tool Examples

```bash
# get-weather (no auth required)
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-weather","arguments":{"city":"Tokyo"}}}'

# get-forecast (requires mcp:read scope)
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer TOKEN_WITH_MCP_READ" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get-forecast","arguments":{"city":"Paris"}}}'

# get-weather-alerts (requires mcp:admin scope)
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer TOKEN_WITH_MCP_ADMIN" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get-weather-alerts","arguments":{"region":"California"}}}'
```

## Troubleshooting

### "Native library not found" at runtime

The native gopher-orch library is required for JWT validation:

```bash
# Download using the run script
./run_example.sh

# Or manually download
curl -sSL https://raw.githubusercontent.com/GopherSecurity/gopher-mcp-rust/main/install-native.sh | bash -s -- latest ./native
```

Verify the library is installed:
```bash
ls -la ./native/lib/libgopher-orch*
```

### Library path not set

```bash
# macOS
export DYLD_LIBRARY_PATH="./native/lib:$DYLD_LIBRARY_PATH"

# Linux
export LD_LIBRARY_PATH="./native/lib:$LD_LIBRARY_PATH"
```

### "Auth client creation failed"

Check that:
1. `jwks_uri` points to a valid JWKS endpoint
2. `issuer` matches the token issuer
3. Network can reach the auth server

### "Token validation failed"

Ensure:
1. Token is not expired
2. Token issuer matches config
3. Token was signed by a key in JWKS
4. Required scopes are present in token

## Project Structure

```
auth/
├── Cargo.toml           # Dependencies (uses gopher-orch SDK)
├── Cargo.lock           # Dependency lock file
├── server.config        # Example configuration
├── run_example.sh       # Build and run script
├── README.md            # This file
├── native/              # Downloaded native libraries
│   ├── lib/             # .dylib/.so files
│   └── include/         # Header files
└── src/
    ├── main.rs          # Entry point and router setup
    ├── config.rs        # Configuration parsing
    ├── cors.rs          # CORS utilities
    ├── error.rs         # Error types
    ├── ffi/
    │   ├── mod.rs       # FFI module
    │   └── auth.rs      # gopher-auth bindings
    ├── middleware/
    │   ├── mod.rs       # Middleware module
    │   └── oauth_auth.rs # Auth middleware
    ├── routes/
    │   ├── mod.rs       # Routes module
    │   ├── health.rs    # Health endpoint
    │   ├── mcp_handler.rs # MCP JSON-RPC handler
    │   └── oauth_endpoints.rs # OAuth discovery
    └── tools/
        ├── mod.rs       # Tools module
        └── weather_tools.rs # Weather tool implementations
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with verbose output:

```bash
cargo test -- --nocapture
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Log level (e.g., `info`, `debug`, `trace`) |
| `SDK_VERSION` | Version of gopher-mcp-rust SDK (default: v0.1.2) |
| `NATIVE_LIB_DIR` | Directory for native libraries (default: ./native/lib) |
| `DYLD_LIBRARY_PATH` | macOS library search path |
| `LD_LIBRARY_PATH` | Linux library search path |

## SDK Documentation

For more information about the gopher-mcp-rust SDK:

- Repository: https://github.com/GopherSecurity/gopher-mcp-rust
- Documentation: https://docs.rs/gopher-orch (after crates.io publish)

## License

MIT License - see LICENSE file for details.

# Rust Auth MCP Server

An OAuth-protected MCP (Model Context Protocol) server example demonstrating JWT token validation and scope-based access control for MCP tools.

## Features

- **OAuth 2.0 Protected Resources**: Implements RFC 9728 protected resource metadata
- **OpenID Connect Discovery**: Supports OIDC discovery endpoints
- **JWT Token Validation**: Validates tokens using gopher-auth native library
- **Scope-Based Access Control**: Tools require specific scopes (e.g., `mcp:read`, `mcp:admin`)
- **MCP Protocol Support**: Full JSON-RPC 2.0 implementation for MCP tools
- **Weather Tools Example**: Three tools demonstrating different access levels

## Requirements

- Rust 1.70 or later
- (Optional) gopher-auth native library for JWT validation

## Quick Start

### Run Without Authentication

The fastest way to try the server:

```bash
./run_example.sh --no-auth
```

Or manually:

```bash
cargo run
```

### Run With Configuration

```bash
./run_example.sh --config server.config
```

### Build Release

```bash
cargo build --release
./target/release/auth-mcp-server server.config
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

## API Endpoints

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

# Call Tool
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get-weather","arguments":{"city":"NYC"}}}'
```

## Available Tools

| Tool | Scope Required | Description |
|------|----------------|-------------|
| `get-weather` | None | Get current weather for a city |
| `get-forecast` | `mcp:read` | Get 5-day weather forecast |
| `get-weather-alerts` | `mcp:admin` | Get weather alerts for a region |

## Project Structure

```
examples/auth/
├── Cargo.toml           # Dependencies and metadata
├── server.config        # Default configuration
├── run_example.sh       # Launcher script
├── README.md            # This file
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

## License

MIT License - see LICENSE file for details.

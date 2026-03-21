# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of gopher-mcp-rust SDK
- Rust bindings for gopher-orch native library via FFI
- Runtime library loading using `libloading` crate
- OAuth 2.0 authentication support (feature-gated with `auth` feature)
- MCP (Model Context Protocol) client implementation
- GopherAgent for AI agent orchestration
- ConfigBuilder for client configuration
- Auth example server with Axum web framework

### Features
- `default` - Core functionality without auth
- `auth` - OAuth 2.0 token validation via native library

---

[Unreleased]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/HEAD

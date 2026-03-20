# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2.8] - 2026-03-21

## [0.1.2.7] - 2026-03-21

## [0.1.2.6] - 2026-03-21

## [0.1.2.5] - 2026-03-21

## [0.1.2.4] - 2026-03-21

## [0.1.2.3] - 2026-03-21

## [0.1.2.1] - 2026-03-21

## [0.1.2] - 2026-03-20

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

[Unreleased]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.8...HEAD
[0.1.2.8]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.7...v0.1.2.8
[0.1.2.7]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.6...v0.1.2.7
[0.1.2.6]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.5...v0.1.2.6
[0.1.2.5]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.4...v0.1.2.5
[0.1.2.4]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.3...v0.1.2.4
[0.1.2.3]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2.1...v0.1.2.3
[0.1.2.1]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/v0.1.2...v0.1.2.1
[0.1.2]: https://github.com/GopherSecurity/gopher-mcp-rust/releases/tag/v0.1.2

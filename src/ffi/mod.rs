//! FFI bindings to native Gopher libraries.
//!
//! This module provides safe Rust bindings to:
//! - `gopher-orch` - AI agent orchestration library
//! - `gopher-auth` - OAuth/JWT authentication library (optional, requires `auth` feature)

pub mod orch;

#[cfg(feature = "auth")]
pub mod auth;

// Re-export orch types at module level for backward compatibility
pub use orch::*;

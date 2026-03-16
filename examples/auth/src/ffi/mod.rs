//! FFI bindings module.
//!
//! Re-exports gopher-auth types from the gopher-orch library.

mod auth;

// Re-export types from local wrapper
pub use auth::{GopherAuthClient, TokenPayload, ValidationResult};

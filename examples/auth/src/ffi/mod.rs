//! FFI bindings module.
//!
//! Provides bindings to the gopher-auth native library.

pub mod auth;

// Re-export commonly used types
pub use auth::{GopherAuthClient, TokenPayload, ValidationResult};

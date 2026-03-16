//! Authentication middleware module.
//!
//! Provides OAuth/JWT authentication middleware for protecting routes.

pub mod oauth_auth;

// Re-export commonly used types
pub use oauth_auth::{AuthContext, AuthState, GopherAuthClient, extract_token};

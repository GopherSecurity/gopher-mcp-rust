//! Authentication middleware module.
//!
//! Provides OAuth/JWT authentication middleware for protecting routes.

pub mod oauth_auth;

// Re-export commonly used types
pub use oauth_auth::{
    auth_middleware, cors_preflight_response, extract_token, unauthorized_response, AuthContext,
    AuthState, GopherAuthClient,
};

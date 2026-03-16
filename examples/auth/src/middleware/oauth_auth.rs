//! OAuth Authentication Middleware
//!
//! Provides OAuth/JWT authentication middleware for protecting MCP routes.
//! Handles token extraction, validation, and scope-based access control.

use axum::http::Request;
use std::sync::Arc;

use crate::config::AuthServerConfig;

/// Authentication context from JWT token validation.
///
/// Contains user information extracted from a validated token.
#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    /// User identifier from token subject.
    pub user_id: String,
    /// Space-separated list of scopes.
    pub scopes: String,
    /// Token audience.
    pub audience: String,
    /// Token expiration timestamp (unix seconds).
    pub token_expiry: u64,
    /// Whether the user is authenticated.
    pub authenticated: bool,
}

impl AuthContext {
    /// Check if a specific scope is present.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.split_whitespace().any(|s| s == scope)
    }
}

/// Placeholder type for the gopher-auth client.
///
/// Will be replaced with actual FFI bindings in a later implementation.
pub struct GopherAuthClient;

/// Shared state for the auth middleware.
pub struct AuthState {
    /// Optional auth client (None if auth is disabled).
    pub auth_client: Option<Arc<GopherAuthClient>>,
    /// Server configuration.
    pub config: AuthServerConfig,
}

impl AuthState {
    /// Create a new auth state.
    pub fn new(auth_client: Option<Arc<GopherAuthClient>>, config: AuthServerConfig) -> Self {
        Self {
            auth_client,
            config,
        }
    }

    /// Check if a path requires authentication.
    ///
    /// Returns false for public paths, true for protected paths.
    pub fn requires_auth(&self, path: &str) -> bool {
        // If auth is disabled or no auth client, nothing requires auth
        if self.config.auth_disabled || self.auth_client.is_none() {
            return false;
        }

        // Public paths that never require auth
        let public_prefixes = [
            "/.well-known/",
            "/oauth/",
            "/authorize",
            "/health",
            "/favicon.ico",
        ];

        for prefix in &public_prefixes {
            if path.starts_with(prefix) || path == *prefix {
                return false;
            }
        }

        // Exact match for public paths
        if path == "/health" || path == "/favicon.ico" {
            return false;
        }

        // Protected paths
        let protected_prefixes = ["/mcp", "/rpc", "/events", "/sse"];

        for prefix in &protected_prefixes {
            if path.starts_with(prefix) {
                return true;
            }
        }

        // Default: protected (fail secure)
        true
    }
}

/// Extract bearer token from a request.
///
/// Looks for the token in:
/// 1. Authorization header (Bearer prefix)
/// 2. access_token query parameter (fallback)
///
/// # Arguments
///
/// * `request` - The HTTP request
///
/// # Returns
///
/// The token string if found, None otherwise
pub fn extract_token<B>(request: &Request<B>) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
            // Also try lowercase prefix
            if let Some(token) = auth_str.strip_prefix("bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Try access_token query parameter as fallback
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(token) = pair.strip_prefix("access_token=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn test_auth_context_default() {
        let ctx = AuthContext::default();
        assert_eq!(ctx.user_id, "");
        assert_eq!(ctx.scopes, "");
        assert_eq!(ctx.audience, "");
        assert_eq!(ctx.token_expiry, 0);
        assert!(!ctx.authenticated);
    }

    #[test]
    fn test_auth_context_has_scope() {
        let ctx = AuthContext {
            scopes: "openid profile mcp:read mcp:admin".to_string(),
            ..Default::default()
        };

        assert!(ctx.has_scope("openid"));
        assert!(ctx.has_scope("profile"));
        assert!(ctx.has_scope("mcp:read"));
        assert!(ctx.has_scope("mcp:admin"));
        assert!(!ctx.has_scope("mcp:write"));
        assert!(!ctx.has_scope(""));
    }

    #[test]
    fn test_auth_context_has_scope_empty() {
        let ctx = AuthContext::default();
        assert!(!ctx.has_scope("openid"));
        assert!(!ctx.has_scope(""));
    }

    #[test]
    fn test_auth_state_requires_auth_disabled() {
        let config = AuthServerConfig {
            auth_disabled: true,
            ..Default::default()
        };
        let state = AuthState::new(Some(Arc::new(GopherAuthClient)), config);

        // Nothing requires auth when auth is disabled
        assert!(!state.requires_auth("/mcp"));
        assert!(!state.requires_auth("/rpc"));
        assert!(!state.requires_auth("/health"));
    }

    #[test]
    fn test_auth_state_requires_auth_no_client() {
        let config = AuthServerConfig {
            auth_disabled: false,
            ..Default::default()
        };
        let state = AuthState::new(None, config);

        // Nothing requires auth when no client
        assert!(!state.requires_auth("/mcp"));
        assert!(!state.requires_auth("/rpc"));
    }

    #[test]
    fn test_auth_state_requires_auth_public_paths() {
        let config = AuthServerConfig {
            auth_disabled: false,
            ..Default::default()
        };
        let state = AuthState::new(Some(Arc::new(GopherAuthClient)), config);

        // Public paths don't require auth
        assert!(!state.requires_auth("/.well-known/oauth-protected-resource"));
        assert!(!state.requires_auth("/.well-known/oauth-authorization-server"));
        assert!(!state.requires_auth("/.well-known/openid-configuration"));
        assert!(!state.requires_auth("/oauth/authorize"));
        assert!(!state.requires_auth("/oauth/register"));
        assert!(!state.requires_auth("/health"));
        assert!(!state.requires_auth("/favicon.ico"));
    }

    #[test]
    fn test_auth_state_requires_auth_protected_paths() {
        let config = AuthServerConfig {
            auth_disabled: false,
            ..Default::default()
        };
        let state = AuthState::new(Some(Arc::new(GopherAuthClient)), config);

        // Protected paths require auth
        assert!(state.requires_auth("/mcp"));
        assert!(state.requires_auth("/mcp/messages"));
        assert!(state.requires_auth("/rpc"));
        assert!(state.requires_auth("/events"));
        assert!(state.requires_auth("/sse"));
    }

    #[test]
    fn test_auth_state_requires_auth_unknown_paths() {
        let config = AuthServerConfig {
            auth_disabled: false,
            ..Default::default()
        };
        let state = AuthState::new(Some(Arc::new(GopherAuthClient)), config);

        // Unknown paths default to protected
        assert!(state.requires_auth("/api/unknown"));
        assert!(state.requires_auth("/foo/bar"));
    }

    #[test]
    fn test_extract_token_from_header() {
        let request = Request::builder()
            .uri("/mcp")
            .header("authorization", "Bearer mytoken123")
            .body(())
            .unwrap();

        assert_eq!(extract_token(&request), Some("mytoken123".to_string()));
    }

    #[test]
    fn test_extract_token_from_header_lowercase() {
        let request = Request::builder()
            .uri("/mcp")
            .header("authorization", "bearer mytoken456")
            .body(())
            .unwrap();

        assert_eq!(extract_token(&request), Some("mytoken456".to_string()));
    }

    #[test]
    fn test_extract_token_from_query() {
        let request = Request::builder()
            .uri("/mcp?access_token=querytoken789")
            .body(())
            .unwrap();

        assert_eq!(extract_token(&request), Some("querytoken789".to_string()));
    }

    #[test]
    fn test_extract_token_from_query_with_other_params() {
        let request = Request::builder()
            .uri("/mcp?foo=bar&access_token=querytoken&baz=qux")
            .body(())
            .unwrap();

        assert_eq!(extract_token(&request), Some("querytoken".to_string()));
    }

    #[test]
    fn test_extract_token_header_priority() {
        let request = Request::builder()
            .uri("/mcp?access_token=querytoken")
            .header("authorization", "Bearer headertoken")
            .body(())
            .unwrap();

        // Header takes priority
        assert_eq!(extract_token(&request), Some("headertoken".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        let request = Request::builder().uri("/mcp").body(()).unwrap();

        assert_eq!(extract_token(&request), None);
    }

    #[test]
    fn test_extract_token_invalid_header() {
        let request = Request::builder()
            .uri("/mcp")
            .header("authorization", "Basic dXNlcjpwYXNz")
            .body(())
            .unwrap();

        assert_eq!(extract_token(&request), None);
    }
}

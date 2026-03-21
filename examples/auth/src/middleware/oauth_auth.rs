//! OAuth Authentication Middleware
//!
//! Provides OAuth/JWT authentication middleware for protecting MCP routes.
//! Handles token extraction, validation, and scope-based access control.

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;

use crate::config::AuthServerConfig;
use crate::cors::{
    CORS_ALLOW_HEADERS, CORS_ALLOW_METHODS, CORS_ALLOW_ORIGIN, CORS_EXPOSE_HEADERS, CORS_MAX_AGE,
};
use crate::ffi::GopherAuthClient;

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

/// Create a CORS preflight response.
///
/// Returns 204 No Content with full CORS headers for OPTIONS requests.
pub fn cors_preflight_response() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", CORS_ALLOW_ORIGIN)
        .header("Access-Control-Allow-Methods", CORS_ALLOW_METHODS)
        .header("Access-Control-Allow-Headers", CORS_ALLOW_HEADERS)
        .header("Access-Control-Expose-Headers", CORS_EXPOSE_HEADERS)
        .header("Access-Control-Max-Age", CORS_MAX_AGE)
        .body(Body::empty())
        .unwrap()
}

/// Create an unauthorized response with WWW-Authenticate header.
///
/// Returns 401 Unauthorized with RFC 6750 Bearer scheme header and JSON body.
///
/// # Arguments
///
/// * `config` - Server configuration for resource metadata URL
/// * `error` - OAuth error code (e.g., "invalid_token", "invalid_request")
/// * `description` - Human-readable error description
pub fn unauthorized_response(config: &AuthServerConfig, error: &str, description: &str) -> Response {
    // Build WWW-Authenticate header value per RFC 6750
    let www_authenticate = format!(
        r#"Bearer realm="{server_url}", resource_metadata="{server_url}/.well-known/oauth-protected-resource", scope="{scopes}", error="{error}", error_description="{description}""#,
        server_url = config.server_url,
        scopes = config.allowed_scopes,
        error = error,
        description = description
    );

    let body = json!({
        "error": error,
        "error_description": description
    });

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", www_authenticate)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", CORS_ALLOW_ORIGIN)
        .header("Access-Control-Expose-Headers", CORS_EXPOSE_HEADERS)
        .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
        .unwrap()
}

/// Authentication middleware.
///
/// Validates bearer tokens and injects AuthContext into request extensions.
///
/// # Flow
///
/// 1. OPTIONS requests → CORS preflight response
/// 2. Public paths → pass through
/// 3. Extract token → 401 if missing
/// 4. Validate token (placeholder) → 401 if invalid
/// 5. Inject AuthContext → continue to handler
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Handle CORS preflight
    if request.method() == Method::OPTIONS {
        return Ok(cors_preflight_response());
    }

    let path = request.uri().path().to_string();

    // Check if this path requires authentication
    if !state.requires_auth(&path) {
        // Insert default auth context for public paths
        request.extensions_mut().insert(AuthContext::default());
        return Ok(next.run(request).await);
    }

    // Extract bearer token
    let token = match extract_token(&request) {
        Some(t) => t,
        None => {
            return Ok(unauthorized_response(
                &state.config,
                "invalid_request",
                "Missing bearer token",
            ));
        }
    };

    // TODO: Validate token using gopher-auth FFI
    // For now, create a mock auth context if a token is present
    // In the real implementation, this would call:
    // - state.auth_client.validate_token(&token, clock_skew)
    // - state.auth_client.extract_payload(&token)

    // Placeholder: accept any token and extract mock claims
    // This will be replaced with actual validation in the FFI implementation
    let auth_context = if state.config.auth_disabled {
        // Auth disabled: grant full access
        AuthContext {
            user_id: "anonymous".to_string(),
            scopes: state.config.allowed_scopes.clone(),
            audience: state.config.server_url.clone(),
            token_expiry: u64::MAX,
            authenticated: false,
        }
    } else {
        // Placeholder for real token validation
        // In production, this would parse and validate the JWT
        AuthContext {
            user_id: "user".to_string(),
            scopes: state.config.allowed_scopes.clone(),
            audience: state.config.server_url.clone(),
            token_expiry: chrono::Utc::now().timestamp() as u64 + 3600,
            authenticated: true,
        }
    };

    // Insert auth context into request extensions
    request.extensions_mut().insert(auth_context);

    // Continue to the next handler
    Ok(next.run(request).await)
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
        let state = AuthState::new(Some(Arc::new(GopherAuthClient::dummy())), config);

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
        let state = AuthState::new(Some(Arc::new(GopherAuthClient::dummy())), config);

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
        let state = AuthState::new(Some(Arc::new(GopherAuthClient::dummy())), config);

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
        let state = AuthState::new(Some(Arc::new(GopherAuthClient::dummy())), config);

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

    #[test]
    fn test_cors_preflight_response_status() {
        let response = cors_preflight_response();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_cors_preflight_response_headers() {
        let response = cors_preflight_response();
        let headers = response.headers();

        assert!(headers.get("Access-Control-Allow-Origin").is_some());
        assert!(headers.get("Access-Control-Allow-Methods").is_some());
        assert!(headers.get("Access-Control-Allow-Headers").is_some());
        assert!(headers.get("Access-Control-Expose-Headers").is_some());
        assert!(headers.get("Access-Control-Max-Age").is_some());
    }

    #[test]
    fn test_unauthorized_response_status() {
        let config = AuthServerConfig {
            server_url: "http://localhost:3001".to_string(),
            allowed_scopes: "openid mcp:read".to_string(),
            ..Default::default()
        };

        let response = unauthorized_response(&config, "invalid_token", "Token expired");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_unauthorized_response_www_authenticate() {
        let config = AuthServerConfig {
            server_url: "http://localhost:3001".to_string(),
            allowed_scopes: "openid mcp:read".to_string(),
            ..Default::default()
        };

        let response = unauthorized_response(&config, "invalid_token", "Token expired");
        let www_auth = response.headers().get("WWW-Authenticate").unwrap().to_str().unwrap();

        assert!(www_auth.contains("Bearer"));
        assert!(www_auth.contains("realm="));
        assert!(www_auth.contains("resource_metadata="));
        assert!(www_auth.contains("error=\"invalid_token\""));
        assert!(www_auth.contains("error_description=\"Token expired\""));
    }

    #[test]
    fn test_unauthorized_response_cors_headers() {
        let config = AuthServerConfig::default();
        let response = unauthorized_response(&config, "invalid_request", "Missing token");

        assert!(response.headers().get("Access-Control-Allow-Origin").is_some());
        assert!(response.headers().get("Access-Control-Expose-Headers").is_some());
    }

    #[test]
    fn test_unauthorized_response_content_type() {
        let config = AuthServerConfig::default();
        let response = unauthorized_response(&config, "invalid_request", "Missing token");

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }
}

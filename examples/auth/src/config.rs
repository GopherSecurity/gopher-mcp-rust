//! Configuration module for the auth MCP server.
//!
//! Provides INI-style configuration file parsing and server configuration.

use std::collections::HashMap;
use std::path::Path;

use crate::error::AppError;

/// Server configuration for the OAuth-protected MCP server.
#[derive(Debug, Clone)]
pub struct AuthServerConfig {
    // Server settings
    /// Server bind address (e.g., "0.0.0.0")
    pub host: String,
    /// Server port
    pub port: u16,
    /// Public server URL for metadata endpoints
    pub server_url: String,

    // OAuth/IDP settings
    /// Base URL of the authorization server (e.g., Keycloak realm URL)
    pub auth_server_url: String,
    /// JWKS endpoint URL for token validation
    pub jwks_uri: String,
    /// Expected token issuer
    pub issuer: String,
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: String,
    /// Token endpoint URL
    pub token_endpoint: String,

    // Direct OAuth endpoint URLs
    /// Authorization endpoint URL
    pub oauth_authorize_url: String,
    /// Token endpoint URL (alternative to token_endpoint)
    pub oauth_token_url: String,

    // Scopes
    /// Space-separated list of allowed scopes
    pub allowed_scopes: String,

    // Cache settings
    /// JWKS cache duration in seconds
    pub jwks_cache_duration: u32,
    /// Whether to auto-refresh JWKS cache
    pub jwks_auto_refresh: bool,
    /// Request timeout in milliseconds
    pub request_timeout: u32,

    // Auth bypass mode
    /// When true, authentication is disabled
    pub auth_disabled: bool,
}

impl Default for AuthServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3001,
            server_url: "http://localhost:3001".to_string(),
            auth_server_url: String::new(),
            jwks_uri: String::new(),
            issuer: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            token_endpoint: String::new(),
            oauth_authorize_url: String::new(),
            oauth_token_url: String::new(),
            allowed_scopes: "mcp:read mcp:admin".to_string(),
            jwks_cache_duration: 3600,
            jwks_auto_refresh: true,
            request_timeout: 5000,
            auth_disabled: false,
        }
    }
}

impl AuthServerConfig {
    /// Create a default config with authentication disabled.
    ///
    /// Useful for testing and development.
    pub fn default_disabled() -> Self {
        Self {
            auth_disabled: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthServerConfig::default();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3001);
        assert_eq!(config.server_url, "http://localhost:3001");
        assert!(!config.auth_disabled);
        assert_eq!(config.jwks_cache_duration, 3600);
        assert!(config.jwks_auto_refresh);
        assert_eq!(config.request_timeout, 5000);
    }

    #[test]
    fn test_default_disabled() {
        let config = AuthServerConfig::default_disabled();

        assert!(config.auth_disabled);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3001);
    }
}

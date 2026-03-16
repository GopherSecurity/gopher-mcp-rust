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

/// Parse INI-style configuration file content.
///
/// Handles:
/// - Comments (lines starting with `#`)
/// - Empty lines
/// - Values containing `=` characters (splits only on first `=`)
/// - Whitespace trimming for keys and values
pub fn parse_config_file(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split on first '=' only to handle values containing '='
        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();

            if !key.is_empty() {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }

    map
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

    #[test]
    fn test_parse_basic_key_value() {
        let content = "host=localhost\nport=3001";
        let map = parse_config_file(content);

        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("port"), Some(&"3001".to_string()));
    }

    #[test]
    fn test_parse_comments_skipped() {
        let content = "# This is a comment\nhost=localhost\n# Another comment\nport=3001";
        let map = parse_config_file(content);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("port"), Some(&"3001".to_string()));
    }

    #[test]
    fn test_parse_empty_lines_skipped() {
        let content = "host=localhost\n\n\nport=3001\n\n";
        let map = parse_config_file(content);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("port"), Some(&"3001".to_string()));
    }

    #[test]
    fn test_parse_values_with_equals() {
        let content = "auth_url=https://auth.example.com?param=value&other=123";
        let map = parse_config_file(content);

        assert_eq!(
            map.get("auth_url"),
            Some(&"https://auth.example.com?param=value&other=123".to_string())
        );
    }

    #[test]
    fn test_parse_whitespace_trimmed() {
        let content = "  host  =  localhost  \n  port=  3001";
        let map = parse_config_file(content);

        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("port"), Some(&"3001".to_string()));
    }

    #[test]
    fn test_parse_empty_value() {
        let content = "empty_key=";
        let map = parse_config_file(content);

        assert_eq!(map.get("empty_key"), Some(&"".to_string()));
    }
}

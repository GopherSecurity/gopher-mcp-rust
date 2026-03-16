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

    /// Build configuration from a parsed key-value map.
    ///
    /// When `auth_server_url` is provided, missing OAuth endpoints are
    /// automatically derived using standard OpenID Connect paths:
    /// - `jwks_uri` → `{auth_server_url}/protocol/openid-connect/certs`
    /// - `issuer` → `{auth_server_url}`
    /// - `oauth_authorize_url` → `{auth_server_url}/protocol/openid-connect/auth`
    /// - `oauth_token_url` → `{auth_server_url}/protocol/openid-connect/token`
    /// - `token_endpoint` → `{auth_server_url}/protocol/openid-connect/token`
    pub fn build_from_map(map: HashMap<String, String>) -> Result<Self, AppError> {
        let defaults = Self::default();

        // Parse basic fields with defaults
        let host = map.get("host").cloned().unwrap_or(defaults.host);
        let port = map
            .get("port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(defaults.port);
        let server_url = map
            .get("server_url")
            .cloned()
            .unwrap_or_else(|| format!("http://{}:{}", host, port));

        // Get auth server URL for endpoint derivation
        let auth_server_url = map
            .get("auth_server_url")
            .cloned()
            .unwrap_or_default();

        // Derive endpoints from auth_server_url if not explicitly set
        let jwks_uri = map.get("jwks_uri").cloned().unwrap_or_else(|| {
            if auth_server_url.is_empty() {
                String::new()
            } else {
                format!("{}/protocol/openid-connect/certs", auth_server_url)
            }
        });

        let issuer = map.get("issuer").cloned().unwrap_or_else(|| {
            auth_server_url.clone()
        });

        let oauth_authorize_url = map.get("oauth_authorize_url").cloned().unwrap_or_else(|| {
            if auth_server_url.is_empty() {
                String::new()
            } else {
                format!("{}/protocol/openid-connect/auth", auth_server_url)
            }
        });

        let oauth_token_url = map.get("oauth_token_url").cloned().unwrap_or_else(|| {
            if auth_server_url.is_empty() {
                String::new()
            } else {
                format!("{}/protocol/openid-connect/token", auth_server_url)
            }
        });

        let token_endpoint = map.get("token_endpoint").cloned().unwrap_or_else(|| {
            if auth_server_url.is_empty() {
                String::new()
            } else {
                format!("{}/protocol/openid-connect/token", auth_server_url)
            }
        });

        // Parse other OAuth fields
        let client_id = map.get("client_id").cloned().unwrap_or_default();
        let client_secret = map.get("client_secret").cloned().unwrap_or_default();
        let allowed_scopes = map
            .get("allowed_scopes")
            .cloned()
            .unwrap_or(defaults.allowed_scopes);

        // Parse cache settings
        let jwks_cache_duration = map
            .get("jwks_cache_duration")
            .and_then(|s| s.parse().ok())
            .unwrap_or(defaults.jwks_cache_duration);

        let jwks_auto_refresh = map
            .get("jwks_auto_refresh")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(defaults.jwks_auto_refresh);

        let request_timeout = map
            .get("request_timeout")
            .and_then(|s| s.parse().ok())
            .unwrap_or(defaults.request_timeout);

        // Parse auth disabled flag
        let auth_disabled = map
            .get("auth_disabled")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(defaults.auth_disabled);

        Ok(Self {
            host,
            port,
            server_url,
            auth_server_url,
            jwks_uri,
            issuer,
            client_id,
            client_secret,
            token_endpoint,
            oauth_authorize_url,
            oauth_token_url,
            allowed_scopes,
            jwks_cache_duration,
            jwks_auto_refresh,
            request_timeout,
            auth_disabled,
        })
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

    #[test]
    fn test_build_from_map_defaults() {
        let map = HashMap::new();
        let config = AuthServerConfig::build_from_map(map).unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3001);
        assert_eq!(config.server_url, "http://0.0.0.0:3001");
        assert!(!config.auth_disabled);
    }

    #[test]
    fn test_build_from_map_custom_values() {
        let mut map = HashMap::new();
        map.insert("host".to_string(), "127.0.0.1".to_string());
        map.insert("port".to_string(), "8080".to_string());
        map.insert("client_id".to_string(), "my-client".to_string());
        map.insert("auth_disabled".to_string(), "true".to_string());

        let config = AuthServerConfig::build_from_map(map).unwrap();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.server_url, "http://127.0.0.1:8080");
        assert_eq!(config.client_id, "my-client");
        assert!(config.auth_disabled);
    }

    #[test]
    fn test_build_from_map_endpoint_derivation() {
        let mut map = HashMap::new();
        map.insert(
            "auth_server_url".to_string(),
            "https://auth.example.com/realms/test".to_string(),
        );

        let config = AuthServerConfig::build_from_map(map).unwrap();

        assert_eq!(
            config.jwks_uri,
            "https://auth.example.com/realms/test/protocol/openid-connect/certs"
        );
        assert_eq!(
            config.issuer,
            "https://auth.example.com/realms/test"
        );
        assert_eq!(
            config.oauth_authorize_url,
            "https://auth.example.com/realms/test/protocol/openid-connect/auth"
        );
        assert_eq!(
            config.oauth_token_url,
            "https://auth.example.com/realms/test/protocol/openid-connect/token"
        );
        assert_eq!(
            config.token_endpoint,
            "https://auth.example.com/realms/test/protocol/openid-connect/token"
        );
    }

    #[test]
    fn test_build_from_map_explicit_endpoints_override() {
        let mut map = HashMap::new();
        map.insert(
            "auth_server_url".to_string(),
            "https://auth.example.com/realms/test".to_string(),
        );
        map.insert(
            "jwks_uri".to_string(),
            "https://custom.example.com/jwks".to_string(),
        );

        let config = AuthServerConfig::build_from_map(map).unwrap();

        // Explicit value should override derived
        assert_eq!(config.jwks_uri, "https://custom.example.com/jwks");
        // Other endpoints still derived
        assert_eq!(
            config.oauth_authorize_url,
            "https://auth.example.com/realms/test/protocol/openid-connect/auth"
        );
    }

    #[test]
    fn test_build_from_map_cache_settings() {
        let mut map = HashMap::new();
        map.insert("jwks_cache_duration".to_string(), "7200".to_string());
        map.insert("jwks_auto_refresh".to_string(), "false".to_string());
        map.insert("request_timeout".to_string(), "10000".to_string());

        let config = AuthServerConfig::build_from_map(map).unwrap();

        assert_eq!(config.jwks_cache_duration, 7200);
        assert!(!config.jwks_auto_refresh);
        assert_eq!(config.request_timeout, 10000);
    }

    #[test]
    fn test_build_from_map_boolean_parsing() {
        // Test "1" as true
        let mut map = HashMap::new();
        map.insert("auth_disabled".to_string(), "1".to_string());
        let config = AuthServerConfig::build_from_map(map).unwrap();
        assert!(config.auth_disabled);

        // Test "true" as true
        let mut map = HashMap::new();
        map.insert("jwks_auto_refresh".to_string(), "true".to_string());
        let config = AuthServerConfig::build_from_map(map).unwrap();
        assert!(config.jwks_auto_refresh);
    }
}

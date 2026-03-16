//! OAuth discovery endpoints.
//!
//! Implements OAuth 2.0 and OpenID Connect discovery endpoints per RFC specifications:
//! - RFC 9728: Protected Resource Metadata
//! - RFC 8414: Authorization Server Metadata
//! - OpenID Connect Discovery 1.0
//! - RFC 7591: Dynamic Client Registration

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::config::AuthServerConfig;
use crate::cors::with_cors_headers;

/// RFC 9728: Protected Resource Metadata.
///
/// Describes the OAuth 2.0 protected resource and its requirements.
#[derive(Debug, Clone, Serialize)]
pub struct ProtectedResourceMetadata {
    /// The protected resource identifier (URL).
    pub resource: String,
    /// List of authorization server URLs.
    pub authorization_servers: Vec<String>,
    /// Supported OAuth scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    /// Supported bearer token methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_methods_supported: Option<Vec<String>>,
    /// URL to resource documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_documentation: Option<String>,
}

/// RFC 8414: Authorization Server Metadata.
///
/// Describes the OAuth 2.0 authorization server configuration.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationServerMetadata {
    /// Authorization server issuer identifier.
    pub issuer: String,
    /// URL of the authorization endpoint.
    pub authorization_endpoint: String,
    /// URL of the token endpoint.
    pub token_endpoint: String,
    /// URL of the JWKS endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    /// URL of the dynamic client registration endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    /// Supported OAuth scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    /// Supported response types.
    pub response_types_supported: Vec<String>,
    /// Supported grant types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types_supported: Option<Vec<String>>,
    /// Supported token endpoint authentication methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    /// Supported PKCE code challenge methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,
}

/// OpenID Connect Discovery 1.0 Configuration.
///
/// Extends RFC 8414 with OIDC-specific fields.
#[derive(Debug, Clone, Serialize)]
pub struct OpenIDConfiguration {
    /// Base authorization server metadata.
    #[serde(flatten)]
    pub base: AuthorizationServerMetadata,
    /// URL of the userinfo endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,
    /// Supported subject identifier types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_types_supported: Option<Vec<String>>,
    /// Supported ID token signing algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_signing_alg_values_supported: Option<Vec<String>>,
}

/// RFC 7591: Client Registration Response.
///
/// Response returned from dynamic client registration endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct ClientRegistrationResponse {
    /// Assigned client identifier.
    pub client_id: String,
    /// Assigned client secret (if confidential client).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Unix timestamp when client_id was issued.
    pub client_id_issued_at: u64,
    /// Unix timestamp when client_secret expires (0 = never).
    pub client_secret_expires_at: u64,
    /// Registered redirect URIs.
    pub redirect_uris: Vec<String>,
    /// Supported grant types.
    pub grant_types: Vec<String>,
    /// Supported response types.
    pub response_types: Vec<String>,
    /// Token endpoint authentication method.
    pub token_endpoint_auth_method: String,
}

/// Client registration request body.
#[derive(Debug, Clone, Deserialize)]
pub struct ClientRegistrationRequest {
    /// Requested redirect URIs.
    #[serde(default)]
    pub redirect_uris: Vec<String>,
}

/// Parse scopes from a space-separated string.
fn parse_scopes(scopes: &str) -> Vec<String> {
    scopes
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Protected resource metadata endpoint handler.
///
/// Returns RFC 9728 compliant metadata describing this MCP resource.
/// Serves both `/.well-known/oauth-protected-resource` and
/// `/.well-known/oauth-protected-resource/mcp`.
pub async fn protected_resource_metadata(
    State(config): State<Arc<AuthServerConfig>>,
) -> impl IntoResponse {
    let scopes = parse_scopes(&config.allowed_scopes);

    let metadata = ProtectedResourceMetadata {
        resource: format!("{}/mcp", config.server_url),
        authorization_servers: vec![config.server_url.clone()],
        scopes_supported: if scopes.is_empty() {
            None
        } else {
            Some(scopes)
        },
        bearer_methods_supported: Some(vec!["header".to_string(), "query".to_string()]),
        resource_documentation: Some(format!("{}/docs", config.server_url)),
    };

    with_cors_headers(Json(metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protected_resource_metadata_serialization() {
        let metadata = ProtectedResourceMetadata {
            resource: "https://example.com/mcp".to_string(),
            authorization_servers: vec!["https://example.com".to_string()],
            scopes_supported: Some(vec!["mcp:read".to_string(), "mcp:admin".to_string()]),
            bearer_methods_supported: Some(vec!["header".to_string(), "query".to_string()]),
            resource_documentation: Some("https://example.com/docs".to_string()),
        };

        let json = serde_json::to_value(&metadata).unwrap();

        assert_eq!(json["resource"], "https://example.com/mcp");
        assert_eq!(json["authorization_servers"][0], "https://example.com");
        assert_eq!(json["scopes_supported"][0], "mcp:read");
        assert_eq!(json["bearer_methods_supported"][0], "header");
    }

    #[test]
    fn test_protected_resource_metadata_omits_none() {
        let metadata = ProtectedResourceMetadata {
            resource: "https://example.com/mcp".to_string(),
            authorization_servers: vec!["https://example.com".to_string()],
            scopes_supported: None,
            bearer_methods_supported: None,
            resource_documentation: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();

        assert!(!json.contains("scopes_supported"));
        assert!(!json.contains("bearer_methods_supported"));
        assert!(!json.contains("resource_documentation"));
    }

    #[test]
    fn test_authorization_server_metadata_serialization() {
        let metadata = AuthorizationServerMetadata {
            issuer: "https://auth.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            jwks_uri: Some("https://auth.example.com/jwks".to_string()),
            registration_endpoint: Some("https://example.com/oauth/register".to_string()),
            scopes_supported: Some(vec!["openid".to_string(), "profile".to_string()]),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: Some(vec!["authorization_code".to_string()]),
            token_endpoint_auth_methods_supported: Some(vec!["client_secret_post".to_string()]),
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        };

        let json = serde_json::to_value(&metadata).unwrap();

        assert_eq!(json["issuer"], "https://auth.example.com");
        assert_eq!(json["response_types_supported"][0], "code");
        assert_eq!(json["code_challenge_methods_supported"][0], "S256");
    }

    #[test]
    fn test_openid_configuration_flattens_base() {
        let base = AuthorizationServerMetadata {
            issuer: "https://auth.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            jwks_uri: None,
            registration_endpoint: None,
            scopes_supported: None,
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: None,
            token_endpoint_auth_methods_supported: None,
            code_challenge_methods_supported: None,
        };

        let oidc = OpenIDConfiguration {
            base,
            userinfo_endpoint: Some("https://auth.example.com/userinfo".to_string()),
            subject_types_supported: Some(vec!["public".to_string()]),
            id_token_signing_alg_values_supported: Some(vec!["RS256".to_string()]),
        };

        let json = serde_json::to_value(&oidc).unwrap();

        // Base fields should be flattened
        assert_eq!(json["issuer"], "https://auth.example.com");
        assert_eq!(json["authorization_endpoint"], "https://auth.example.com/authorize");
        // OIDC-specific fields
        assert_eq!(json["userinfo_endpoint"], "https://auth.example.com/userinfo");
        assert_eq!(json["subject_types_supported"][0], "public");
        assert_eq!(json["id_token_signing_alg_values_supported"][0], "RS256");
    }

    #[test]
    fn test_client_registration_response_serialization() {
        let response = ClientRegistrationResponse {
            client_id: "test-client".to_string(),
            client_secret: Some("secret".to_string()),
            client_id_issued_at: 1704067200,
            client_secret_expires_at: 0,
            redirect_uris: vec!["https://example.com/callback".to_string()],
            grant_types: vec!["authorization_code".to_string(), "refresh_token".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_post".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["client_id"], "test-client");
        assert_eq!(json["client_secret"], "secret");
        assert_eq!(json["client_id_issued_at"], 1704067200);
        assert_eq!(json["client_secret_expires_at"], 0);
        assert_eq!(json["grant_types"][0], "authorization_code");
    }

    #[test]
    fn test_client_registration_response_omits_none_secret() {
        let response = ClientRegistrationResponse {
            client_id: "public-client".to_string(),
            client_secret: None,
            client_id_issued_at: 1704067200,
            client_secret_expires_at: 0,
            redirect_uris: vec![],
            grant_types: vec!["authorization_code".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "none".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();

        // client_secret field should be absent (not null)
        assert!(json.get("client_secret").is_none());
        // But client_secret_expires_at should still be present
        assert!(json.get("client_secret_expires_at").is_some());
    }

    #[test]
    fn test_client_registration_request_deserialization() {
        let json = r#"{"redirect_uris": ["https://example.com/callback"]}"#;
        let request: ClientRegistrationRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.redirect_uris.len(), 1);
        assert_eq!(request.redirect_uris[0], "https://example.com/callback");
    }

    #[test]
    fn test_client_registration_request_empty() {
        let json = r#"{}"#;
        let request: ClientRegistrationRequest = serde_json::from_str(json).unwrap();

        assert!(request.redirect_uris.is_empty());
    }
}

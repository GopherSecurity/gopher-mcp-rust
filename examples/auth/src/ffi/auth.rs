//! FFI Bindings wrapper for gopher-auth
//!
//! Provides a thin wrapper around the gopher-orch library's auth FFI,
//! converting errors to the local AppError type.

use crate::error::AppError;

// Re-export types from gopher-orch
pub use gopher_orch::TokenPayload;
pub use gopher_orch::ValidationResult;

/// Client wrapper for gopher-auth native library.
///
/// Wraps the gopher-orch library's GopherAuthClient and converts errors
/// to the local AppError type.
pub struct GopherAuthClient {
    inner: Option<gopher_orch::GopherAuthClient>,
}

// Safety: Delegates to inner client which is Send + Sync
unsafe impl Send for GopherAuthClient {}
unsafe impl Sync for GopherAuthClient {}

impl GopherAuthClient {
    /// Create a new gopher-auth client.
    ///
    /// # Arguments
    ///
    /// * `jwks_uri` - URI to fetch JWKS from
    /// * `issuer` - Expected token issuer
    ///
    /// # Returns
    ///
    /// A new client instance or an error if initialization failed.
    pub fn new(jwks_uri: &str, issuer: &str) -> Result<Self, AppError> {
        let inner = gopher_orch::GopherAuthClient::new(jwks_uri, issuer)?;
        Ok(Self { inner: Some(inner) })
    }

    /// Validate a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    /// * `clock_skew` - Allowed clock skew in seconds
    ///
    /// # Returns
    ///
    /// Validation result indicating success or failure.
    pub fn validate_token(&self, token: &str, clock_skew: u32) -> ValidationResult {
        match &self.inner {
            Some(client) => client.validate_token(token, clock_skew),
            None => ValidationResult::failure(-1, "Client not initialized"),
        }
    }

    /// Extract payload from a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    ///
    /// # Returns
    ///
    /// Extracted token payload or an error.
    pub fn extract_payload(&self, token: &str) -> Result<TokenPayload, AppError> {
        match &self.inner {
            Some(client) => client.extract_payload(token).map_err(Into::into),
            None => Err(AppError::Ffi("Client not initialized".to_string())),
        }
    }

    /// Set a client option.
    ///
    /// # Arguments
    ///
    /// * `key` - Option key
    /// * `value` - Option value
    ///
    /// # Returns
    ///
    /// Ok if successful, Err otherwise.
    pub fn set_option(&self, key: &str, value: &str) -> Result<(), AppError> {
        match &self.inner {
            Some(client) => client.set_option(key, value).map_err(Into::into),
            None => Err(AppError::Ffi("Client not initialized".to_string())),
        }
    }

    /// Explicitly destroy the client handle.
    ///
    /// This is called automatically by Drop, but can be called manually
    /// to release resources early.
    pub fn destroy(&mut self) {
        if let Some(ref mut client) = self.inner {
            client.destroy();
        }
        self.inner = None;
    }

    /// Create a dummy client for testing purposes.
    ///
    /// This client has no inner implementation and should only be used in tests
    /// that need to check if a client exists without performing operations.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self { inner: None }
    }
}

impl Drop for GopherAuthClient {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert_eq!(result.error_code, 0);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_validation_result_failure() {
        let result = ValidationResult::failure(-1, "Token expired");
        assert!(!result.valid);
        assert_eq!(result.error_code, -1);
        assert_eq!(result.error_message, Some("Token expired".to_string()));
    }

    #[test]
    fn test_token_payload_fields() {
        let payload = TokenPayload {
            subject: "user123".to_string(),
            scopes: "openid profile".to_string(),
            audience: "my-app".to_string(),
            expiration: 1234567890,
        };

        assert_eq!(payload.subject, "user123");
        assert_eq!(payload.scopes, "openid profile");
        assert_eq!(payload.audience, "my-app");
        assert_eq!(payload.expiration, 1234567890);
    }

    #[test]
    fn test_token_payload_clone() {
        let payload = TokenPayload {
            subject: "user".to_string(),
            scopes: "read write".to_string(),
            audience: "api".to_string(),
            expiration: 9999999999,
        };

        let cloned = payload.clone();
        assert_eq!(payload.subject, cloned.subject);
        assert_eq!(payload.scopes, cloned.scopes);
    }

    #[test]
    fn test_dummy_client() {
        let client = GopherAuthClient::dummy();
        // Dummy client should return failure for validation
        let result = client.validate_token("test", 0);
        assert!(!result.valid);
    }
}

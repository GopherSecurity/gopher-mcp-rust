//! FFI bindings module.
//!
//! Re-exports gopher-auth types from the gopher-mcp-rust library with a thin
//! wrapper to support testing without the native library.

use crate::error::AppError;

// Re-export payload types directly
pub use gopher_mcp_rust::TokenPayload;
pub use gopher_mcp_rust::ValidationResult;

/// Wrapper around gopher-mcp-rust's GopherAuthClient.
///
/// Provides the same interface but allows creating dummy instances for testing.
pub struct GopherAuthClient {
    inner: Option<gopher_mcp_rust::GopherAuthClient>,
}

unsafe impl Send for GopherAuthClient {}
unsafe impl Sync for GopherAuthClient {}

impl GopherAuthClient {
    /// Create a new client.
    pub fn new(jwks_uri: &str, issuer: &str) -> Result<Self, AppError> {
        let inner = gopher_mcp_rust::GopherAuthClient::new(jwks_uri, issuer)?;
        Ok(Self { inner: Some(inner) })
    }

    /// Validate a JWT token.
    pub fn validate_token(&self, token: &str, clock_skew: u32) -> ValidationResult {
        match &self.inner {
            Some(client) => client.validate_token(token, clock_skew),
            None => ValidationResult::failure(-1, "Client not initialized"),
        }
    }

    /// Extract payload from a JWT token.
    pub fn extract_payload(&self, token: &str) -> Result<TokenPayload, AppError> {
        match &self.inner {
            Some(client) => client.extract_payload(token).map_err(Into::into),
            None => Err(AppError::Ffi("Client not initialized".to_string())),
        }
    }

    /// Set a client option.
    pub fn set_option(&self, key: &str, value: &str) -> Result<(), AppError> {
        match &self.inner {
            Some(client) => client.set_option(key, value).map_err(Into::into),
            None => Err(AppError::Ffi("Client not initialized".to_string())),
        }
    }

    /// Destroy the client.
    pub fn destroy(&mut self) {
        if let Some(ref mut client) = self.inner {
            client.destroy();
        }
        self.inner = None;
    }

    /// Create a dummy client for testing.
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

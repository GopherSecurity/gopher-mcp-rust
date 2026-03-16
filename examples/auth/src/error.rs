//! Error types for the auth MCP server.
//!
//! Defines the main error enum with variants for configuration,
//! authentication, FFI, JSON-RPC, and internal errors.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

// Re-export gopher_orch error for convenience
pub use gopher_orch::Error as GopherOrchError;

/// Application error type.
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error.
    #[error("Auth error: {0}")]
    Auth(String),

    /// FFI/native library error.
    #[error("FFI error: {0}")]
    Ffi(String),

    /// JSON-RPC protocol error.
    #[error("JSON-RPC error: {message}")]
    JsonRpc { code: i32, message: String },

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<GopherOrchError> for AppError {
    fn from(err: GopherOrchError) -> Self {
        match err {
            GopherOrchError::Auth(msg) => AppError::Auth(msg),
            GopherOrchError::Library(msg) => AppError::Ffi(msg),
            other => AppError::Ffi(other.to_string()),
        }
    }
}

/// JSON error response body.
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            AppError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: msg.clone(),
                    code: None,
                },
            ),
            AppError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: msg.clone(),
                    code: None,
                },
            ),
            AppError::Ffi(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: msg.clone(),
                    code: None,
                },
            ),
            AppError::JsonRpc { code, message } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: message.clone(),
                    code: Some(*code),
                },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: msg.clone(),
                    code: None,
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_config_error_response() {
        let error = AppError::Config("missing field".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "missing field");
        assert!(json.get("code").is_none() || json["code"].is_null());
    }

    #[tokio::test]
    async fn test_auth_error_response() {
        let error = AppError::Auth("invalid token".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "invalid token");
    }

    #[tokio::test]
    async fn test_ffi_error_response() {
        let error = AppError::Ffi("library not found".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "library not found");
    }

    #[tokio::test]
    async fn test_jsonrpc_error_response() {
        let error = AppError::JsonRpc {
            code: -32600,
            message: "Invalid Request".to_string(),
        };
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "Invalid Request");
        assert_eq!(json["code"], -32600);
    }

    #[tokio::test]
    async fn test_internal_error_response() {
        let error = AppError::Internal("unexpected error".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "unexpected error");
    }

    #[test]
    fn test_error_display() {
        let config_err = AppError::Config("test config".to_string());
        assert_eq!(format!("{}", config_err), "Configuration error: test config");

        let auth_err = AppError::Auth("test auth".to_string());
        assert_eq!(format!("{}", auth_err), "Auth error: test auth");

        let ffi_err = AppError::Ffi("test ffi".to_string());
        assert_eq!(format!("{}", ffi_err), "FFI error: test ffi");

        let jsonrpc_err = AppError::JsonRpc {
            code: -32700,
            message: "Parse error".to_string(),
        };
        assert_eq!(format!("{}", jsonrpc_err), "JSON-RPC error: Parse error");

        let internal_err = AppError::Internal("test internal".to_string());
        assert_eq!(format!("{}", internal_err), "Internal error: test internal");
    }
}

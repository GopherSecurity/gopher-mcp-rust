//! CORS utilities for the auth MCP server.
//!
//! Provides CORS headers and handlers matching the TypeScript implementation
//! for browser compatibility with MCP clients.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// CORS allowed origin (permissive for MCP clients).
pub const CORS_ALLOW_ORIGIN: &str = "*";

/// CORS allowed methods.
pub const CORS_ALLOW_METHODS: &str = "GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD";

/// CORS allowed headers (includes MCP-specific headers).
pub const CORS_ALLOW_HEADERS: &str = "Accept, Accept-Language, Content-Language, Content-Type, \
    Authorization, X-Requested-With, Origin, Cache-Control, Pragma, \
    Mcp-Session-Id, Mcp-Protocol-Version";

/// CORS exposed headers.
pub const CORS_EXPOSE_HEADERS: &str = "WWW-Authenticate, Content-Length, Content-Type";

/// CORS max age in seconds (24 hours).
pub const CORS_MAX_AGE: &str = "86400";

/// OPTIONS handler for CORS preflight requests.
///
/// Returns 204 No Content with all CORS headers set.
pub async fn options_handler() -> impl IntoResponse {
    (
        StatusCode::NO_CONTENT,
        [
            ("Access-Control-Allow-Origin", CORS_ALLOW_ORIGIN),
            ("Access-Control-Allow-Methods", CORS_ALLOW_METHODS),
            ("Access-Control-Allow-Headers", CORS_ALLOW_HEADERS),
            ("Access-Control-Expose-Headers", CORS_EXPOSE_HEADERS),
            ("Access-Control-Max-Age", CORS_MAX_AGE),
            ("Content-Length", "0"),
        ],
    )
}

/// Add CORS headers to a response.
///
/// Wraps any response type and adds the necessary CORS headers
/// for cross-origin requests.
pub fn with_cors_headers<T: IntoResponse>(response: T) -> Response {
    let mut res = response.into_response();
    let headers = res.headers_mut();

    headers.insert(
        "Access-Control-Allow-Origin",
        CORS_ALLOW_ORIGIN.parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        CORS_ALLOW_METHODS.parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        CORS_ALLOW_HEADERS.parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        CORS_EXPOSE_HEADERS.parse().unwrap(),
    );
    headers.insert("Access-Control-Max-Age", CORS_MAX_AGE.parse().unwrap());

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_options_handler_status() {
        let response = options_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_options_handler_headers() {
        let response = options_handler().await.into_response();
        let headers = response.headers();

        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods").unwrap(),
            CORS_ALLOW_METHODS
        );
        assert!(headers
            .get("Access-Control-Allow-Headers")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Mcp-Session-Id"));
        assert!(headers
            .get("Access-Control-Allow-Headers")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Mcp-Protocol-Version"));
        assert_eq!(
            headers.get("Access-Control-Max-Age").unwrap(),
            "86400"
        );
        assert_eq!(headers.get("Content-Length").unwrap(), "0");
    }

    #[tokio::test]
    async fn test_options_handler_empty_body() {
        let response = options_handler().await.into_response();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn test_with_cors_headers() {
        let original = (StatusCode::OK, "Hello");
        let response = with_cors_headers(original);

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods").unwrap(),
            CORS_ALLOW_METHODS
        );
        assert_eq!(
            headers.get("Access-Control-Max-Age").unwrap(),
            "86400"
        );
    }

    #[tokio::test]
    async fn test_with_cors_headers_preserves_body() {
        use axum::Json;
        use serde_json::json;

        let original = Json(json!({"message": "test"}));
        let response = with_cors_headers(original);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], "test");
    }

    #[test]
    fn test_cors_headers_include_mcp_headers() {
        // Verify MCP-specific headers are in the allowed list
        assert!(CORS_ALLOW_HEADERS.contains("Mcp-Session-Id"));
        assert!(CORS_ALLOW_HEADERS.contains("Mcp-Protocol-Version"));
    }
}

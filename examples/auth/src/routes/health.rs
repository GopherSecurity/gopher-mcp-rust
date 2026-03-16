//! Health check endpoint.
//!
//! Provides a simple health endpoint for monitoring server status.

use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Server status (always "ok" when responding).
    status: String,
    /// ISO 8601 timestamp.
    timestamp: String,
    /// Optional server version.
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    /// Optional uptime in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    uptime: Option<u64>,
}

/// Shared state for the health endpoint.
pub struct HealthState {
    /// Server start time for uptime calculation.
    start_time: Instant,
    /// Optional version string.
    version: Option<String>,
}

impl HealthState {
    /// Create a new health state with the given version.
    pub fn new(version: Option<String>) -> Self {
        Self {
            start_time: Instant::now(),
            version,
        }
    }
}

/// Health endpoint handler.
///
/// Returns JSON with server status, timestamp, optional version, and uptime.
pub async fn health_handler(State(state): State<Arc<HealthState>>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: state.version.clone(),
        uptime: Some(uptime),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_health_response_format() {
        let state = Arc::new(HealthState::new(Some("1.0.0".to_string())));
        let response = health_handler(State(state)).await.into_response();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
        assert!(json["timestamp"].is_string());
        assert_eq!(json["version"], "1.0.0");
        assert!(json["uptime"].is_number());
    }

    #[tokio::test]
    async fn test_health_response_without_version() {
        let state = Arc::new(HealthState::new(None));
        let response = health_handler(State(state)).await.into_response();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
        // Version should be absent (not null) due to skip_serializing_if
        assert!(json.get("version").is_none());
    }

    #[tokio::test]
    async fn test_health_state_uptime() {
        let state = HealthState::new(None);

        // Wait a tiny bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        let elapsed = state.start_time.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "ok".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            version: Some("1.0.0".to_string()),
            uptime: Some(100),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"uptime\":100"));
    }

    #[test]
    fn test_health_response_omits_none_fields() {
        let response = HealthResponse {
            status: "ok".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            version: None,
            uptime: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("version"));
        assert!(!json.contains("uptime"));
    }
}

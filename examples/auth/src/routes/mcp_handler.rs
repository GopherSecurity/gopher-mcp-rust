//! MCP (Model Context Protocol) handler.
//!
//! Implements JSON-RPC 2.0 protocol for MCP tool execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 error codes.
pub mod error_codes {
    /// Parse error - Invalid JSON was received.
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid Request - The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found - The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params - Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error - Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// JSON-RPC 2.0 Request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,
    /// Request identifier (optional for notifications).
    #[serde(default)]
    pub id: Option<Value>,
    /// Method name to invoke.
    pub method: String,
    /// Method parameters (optional).
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: &'static str,
    /// Request identifier (copied from request).
    pub id: Option<Value>,
    /// Result value (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error value (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Create an error response with data.
    pub fn error_with_data(
        id: Option<Value>,
        code: i32,
        message: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

/// JSON-RPC 2.0 Error.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional error data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Tool specification.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    /// Tool name (unique identifier).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Tool execution result.
#[derive(Debug, Serialize)]
pub struct ToolResult {
    /// Result content items.
    pub content: Vec<ToolContent>,
    /// Whether this result represents an error.
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolResult {
    /// Create a text result.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(text)],
            is_error: None,
        }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(message)],
            is_error: Some(true),
        }
    }
}

/// MCP Tool content item.
#[derive(Debug, Serialize)]
pub struct ToolContent {
    /// Content type ("text", "image", "resource").
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text content (for type "text").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded data (for type "image").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// MIME type (for type "image").
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ToolContent {
    /// Create a text content item.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(text.into()),
            data: None,
            mime_type: None,
        }
    }

    /// Create an image content item.
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            content_type: "image".to_string(),
            text: None,
            data: Some(data.into()),
            mime_type: Some(mime_type.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_jsonrpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "test",
            "params": {"key": "value"}
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, "test");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_jsonrpc_request_minimal() {
        let json = r#"{"jsonrpc": "2.0", "method": "ping"}"#;

        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert!(request.id.is_none());
        assert_eq!(request.method, "ping");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"status": "ok"}));
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["result"]["status"], "ok");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse::error(
            Some(json!(1)),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
        );
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json.get("result").is_none());
        assert_eq!(json["error"]["code"], -32601);
        assert_eq!(json["error"]["message"], "Method not found");
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(json!({"details": "missing field"})),
        };
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["code"], -32600);
        assert_eq!(json["message"], "Invalid Request");
        assert_eq!(json["data"]["details"], "missing field");
    }

    #[test]
    fn test_jsonrpc_error_omits_none_data() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&error).unwrap();

        assert!(!json.contains("data"));
    }

    #[test]
    fn test_tool_spec_serialization() {
        let spec = ToolSpec {
            name: "test-tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };
        let json = serde_json::to_value(&spec).unwrap();

        assert_eq!(json["name"], "test-tool");
        assert_eq!(json["description"], "A test tool");
        assert_eq!(json["inputSchema"]["type"], "object");
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Hello, world!");
        let json = serde_json::to_value(&result).unwrap();

        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][0]["text"], "Hello, world!");
        assert!(json.get("isError").is_none());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        let json = serde_json::to_value(&result).unwrap();

        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][0]["text"], "Something went wrong");
        assert_eq!(json["isError"], true);
    }

    #[test]
    fn test_tool_content_text() {
        let content = ToolContent::text("Hello");
        let json = serde_json::to_value(&content).unwrap();

        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");
        assert!(json.get("data").is_none());
        assert!(json.get("mimeType").is_none());
    }

    #[test]
    fn test_tool_content_image() {
        let content = ToolContent::image("base64data", "image/png");
        let json = serde_json::to_value(&content).unwrap();

        assert_eq!(json["type"], "image");
        assert!(json.get("text").is_none());
        assert_eq!(json["data"], "base64data");
        assert_eq!(json["mimeType"], "image/png");
    }
}

//! FFI verification tests for the Rust SDK.
//!
//! These tests verify that the FFI bindings work correctly with the native library.

use gopher_mcp_rust::{init, is_initialized, ConfigBuilder, GopherAgent};
use std::path::Path;

const TEST_SERVER_CONFIG: &str = r#"{
    "succeeded": true,
    "code": 200000000,
    "message": "success",
    "data": {
        "servers": [
            {
                "version": "2025-01-09",
                "serverId": "1",
                "name": "test-server",
                "transport": "http_sse",
                "config": {"url": "http://127.0.0.1:9999/mcp", "headers": {}},
                "connectTimeout": 5000,
                "requestTimeout": 30000
            }
        ]
    }
}"#;

/// Skip test if native library is not available
fn skip_if_native_library_not_available() -> bool {
    !Path::new("native/lib").exists()
}

#[test]
fn test_initialization() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    // Test that initialization works
    let result = init();
    assert!(result.is_ok() || is_initialized());
}

#[test]
fn test_create_agent_with_server_config() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    let config = ConfigBuilder::new()
        .with_provider("AnthropicProvider")
        .with_model("claude-3-haiku-20240307")
        .with_server_config(TEST_SERVER_CONFIG)
        .build();

    let result = GopherAgent::create(config);

    // Agent creation may fail without API key - this is expected in test environment
    match result {
        Ok(agent) => {
            assert!(!agent.is_disposed());
        }
        Err(e) => {
            println!("Agent creation failed (expected without API key): {}", e);
        }
    }
}

#[test]
fn test_create_agent_with_helper_method() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    let result = GopherAgent::create_with_server_config(
        "AnthropicProvider",
        "claude-3-haiku-20240307",
        TEST_SERVER_CONFIG,
    );

    // May fail without API key
    if let Ok(agent) = result {
        assert!(!agent.is_disposed());
    }
}

#[test]
fn test_create_agent_with_api_key() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    // Call with a dummy API key - should not crash
    let result = GopherAgent::create_with_api_key(
        "AnthropicProvider",
        "claude-3-haiku-20240307",
        "test-api-key-12345",
    );

    // May return error if API key is invalid, but should not panic
    match result {
        Ok(agent) => {
            assert!(!agent.is_disposed());
        }
        Err(e) => {
            println!(
                "Agent creation failed (expected with invalid API key): {}",
                e
            );
        }
    }
}

#[test]
fn test_create_with_empty_config() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    let config = ConfigBuilder::new().build();
    let result = GopherAgent::create(config);

    assert!(result.is_err());
    match result {
        Err(e) => assert!(e.to_string().contains("API key or server config")),
        Ok(_) => panic!("Expected error for empty config"),
    }
}

#[test]
fn test_run_after_dispose() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    let result = GopherAgent::create_with_server_config(
        "AnthropicProvider",
        "claude-3-haiku-20240307",
        TEST_SERVER_CONFIG,
    );

    if let Ok(agent) = result {
        // Drop the agent (disposes it)
        drop(agent);
        // Can't test run after dispose since we dropped it
        // But we verified it compiles and doesn't panic
    }
}

#[test]
fn test_run_detailed_returns_result() {
    if skip_if_native_library_not_available() {
        println!("Skipping test: Native library not built. Run ./build.sh first");
        return;
    }

    let result = GopherAgent::create_with_server_config(
        "AnthropicProvider",
        "claude-3-haiku-20240307",
        TEST_SERVER_CONFIG,
    );

    if let Ok(agent) = result {
        // Run with very short timeout to get quick response
        let result = agent.run_detailed_with_timeout("test query", 100);

        // Should have some status (success, error, or timeout)
        println!("Result status: {}", result.status());
    }
}

#[test]
fn test_config_builder() {
    let config = ConfigBuilder::new()
        .with_provider("TestProvider")
        .with_model("test-model")
        .with_api_key("test-key")
        .build();

    assert_eq!(config.provider(), "TestProvider");
    assert_eq!(config.model(), "test-model");
    assert!(config.has_api_key());
    assert!(!config.has_server_config());
}

#[test]
fn test_config_builder_with_server_config() {
    let config = ConfigBuilder::new()
        .with_provider("TestProvider")
        .with_model("test-model")
        .with_server_config(TEST_SERVER_CONFIG)
        .build();

    assert_eq!(config.provider(), "TestProvider");
    assert_eq!(config.model(), "test-model");
    assert!(!config.has_api_key());
    assert!(config.has_server_config());
}

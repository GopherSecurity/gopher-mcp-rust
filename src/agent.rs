//! GopherAgent implementation.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::ffi::{self, AgentHandle};
use crate::result::AgentResult;
use crate::init;

/// Default timeout for agent queries (60 seconds).
const DEFAULT_TIMEOUT_MS: u64 = 60_000;

/// A gopher-orch agent for running AI queries.
pub struct GopherAgent {
    handle: AgentHandle,
    disposed: AtomicBool,
}

// Safety: The native handle is thread-safe according to the C++ implementation
unsafe impl Send for GopherAgent {}
unsafe impl Sync for GopherAgent {}

impl GopherAgent {
    /// Create a new GopherAgent with the given configuration.
    pub fn create(config: Config) -> Result<Self> {
        init()?;

        let handle = if config.has_api_key() {
            ffi::agent_create_by_api_key(
                config.provider(),
                config.model(),
                config.api_key().unwrap_or(""),
            )
        } else if config.has_server_config() {
            ffi::agent_create_by_json(
                config.provider(),
                config.model(),
                config.server_config().unwrap_or(""),
            )
        } else {
            return Err(Error::config("Either API key or server config must be provided"));
        };

        if handle.is_null() {
            let err_msg = ffi::get_last_error();
            ffi::clear_error();
            let msg = if err_msg.is_empty() {
                "Failed to create agent".to_string()
            } else {
                err_msg
            };
            return Err(Error::agent(msg));
        }

        Ok(GopherAgent {
            handle,
            disposed: AtomicBool::new(false),
        })
    }

    /// Create a new GopherAgent with an API key.
    pub fn create_with_api_key(provider: &str, model: &str, api_key: &str) -> Result<Self> {
        let config = crate::ConfigBuilder::new()
            .with_provider(provider)
            .with_model(model)
            .with_api_key(api_key)
            .build();
        Self::create(config)
    }

    /// Create a new GopherAgent with a server config.
    pub fn create_with_server_config(provider: &str, model: &str, server_config: &str) -> Result<Self> {
        let config = crate::ConfigBuilder::new()
            .with_provider(provider)
            .with_model(model)
            .with_server_config(server_config)
            .build();
        Self::create(config)
    }

    /// Run a query against the agent with the default timeout (60 seconds).
    pub fn run(&self, query: &str) -> Result<String> {
        self.run_with_timeout(query, DEFAULT_TIMEOUT_MS)
    }

    /// Run a query against the agent with a custom timeout.
    pub fn run_with_timeout(&self, query: &str, timeout_ms: u64) -> Result<String> {
        self.ensure_not_disposed()?;

        let response = ffi::agent_run(self.handle, query, timeout_ms);
        if response.is_empty() {
            Ok(format!("No response for query: \"{}\"", query))
        } else {
            Ok(response)
        }
    }

    /// Run a query and return detailed result information.
    pub fn run_detailed(&self, query: &str) -> AgentResult {
        self.run_detailed_with_timeout(query, DEFAULT_TIMEOUT_MS)
    }

    /// Run a query with custom timeout and return detailed result.
    pub fn run_detailed_with_timeout(&self, query: &str, timeout_ms: u64) -> AgentResult {
        match self.run_with_timeout(query, timeout_ms) {
            Ok(response) => AgentResult::success(response),
            Err(Error::Timeout(msg)) => AgentResult::timeout(msg),
            Err(e) => AgentResult::error(e.to_string()),
        }
    }

    /// Check if the agent has been disposed.
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    /// Ensure the agent has not been disposed.
    fn ensure_not_disposed(&self) -> Result<()> {
        if self.is_disposed() {
            Err(Error::Disposed)
        } else {
            Ok(())
        }
    }

    /// Dispose of the agent, releasing native resources.
    fn dispose(&self) {
        if self.disposed.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            if !self.handle.is_null() {
                ffi::agent_release(self.handle);
            }
        }
    }
}

impl Drop for GopherAgent {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn skip_if_native_library_not_available() -> bool {
        !ffi::is_available()
    }

    #[test]
    fn test_create_with_empty_config() {
        if skip_if_native_library_not_available() {
            return;
        }

        let config = crate::ConfigBuilder::new().build();
        let result = GopherAgent::create(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_with_server_config() {
        if skip_if_native_library_not_available() {
            return;
        }

        let result = GopherAgent::create_with_server_config(
            "AnthropicProvider",
            "claude-3-haiku-20240307",
            TEST_SERVER_CONFIG,
        );

        // May fail without API key, but should not panic
        if let Ok(agent) = result {
            assert!(!agent.is_disposed());
        }
    }

    #[test]
    fn test_disposed_after_drop() {
        if skip_if_native_library_not_available() {
            return;
        }

        let result = GopherAgent::create_with_server_config(
            "AnthropicProvider",
            "claude-3-haiku-20240307",
            TEST_SERVER_CONFIG,
        );

        if let Ok(agent) = result {
            assert!(!agent.is_disposed());
            drop(agent);
            // Agent is now disposed (dropped)
        }
    }
}

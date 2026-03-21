//! # Gopher MCP Rust SDK
//!
//! Rust SDK for Gopher Orch - AI Agent orchestration framework with native C++ performance.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use gopher_mcp_rust::{GopherAgent, ConfigBuilder};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create agent with server configuration
//!     let config = ConfigBuilder::new()
//!         .with_provider("AnthropicProvider")
//!         .with_model("claude-3-haiku-20240307")
//!         .with_server_config(r#"{"succeeded": true, "data": {"servers": []}}"#)
//!         .build();
//!
//!     let agent = GopherAgent::create(config)?;
//!
//!     // Run a query
//!     let result = agent.run("What is the weather in Tokyo?")?;
//!     println!("{}", result);
//!
//!     Ok(())
//! }
//! ```

mod agent;
mod config;
mod error;
mod ffi;
mod result;

pub use agent::GopherAgent;
pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use result::{AgentResult, AgentResultStatus};

// Re-export auth types when the auth feature is enabled
#[cfg(feature = "auth")]
pub use ffi::auth::{GopherAuthClient, TokenPayload, ValidationResult};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

static INIT: Once = Once::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the gopher-orch library.
/// Called automatically by `GopherAgent::create()` if not already initialized.
pub fn init() -> Result<()> {
    let mut init_result = Ok(());

    INIT.call_once(|| {
        if !ffi::is_available() {
            init_result = Err(Error::Library(
                "Failed to load gopher-orch native library".into(),
            ));
            return;
        }
        INITIALIZED.store(true, Ordering::SeqCst);
    });

    init_result
}

/// Returns true if the library is initialized.
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// Shutdown the gopher-orch library.
pub fn shutdown() {
    INITIALIZED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Just test that init doesn't panic
        let _ = init();
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
    }
}

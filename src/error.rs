//! Error types for the gopher-orch SDK.

use thiserror::Error;

/// Result type alias for gopher-orch operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for gopher-orch operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Error loading or using the native library.
    #[error("Library error: {0}")]
    Library(String),

    /// Error creating an agent.
    #[error("Agent error: {0}")]
    Agent(String),

    /// Invalid API key error.
    #[error("API key error: {0}")]
    ApiKey(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Timeout error.
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Agent has been disposed.
    #[error("Agent has been disposed")]
    Disposed,
}

impl Error {
    /// Create a new agent error.
    pub fn agent<S: Into<String>>(msg: S) -> Self {
        Error::Agent(msg.into())
    }

    /// Create a new library error.
    pub fn library<S: Into<String>>(msg: S) -> Self {
        Error::Library(msg.into())
    }

    /// Create a new connection error.
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Error::Connection(msg.into())
    }

    /// Create a new timeout error.
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create a new config error.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }
}

//! Error types for the gopher-orch SDK.

use std::error::Error as StdError;
use std::fmt;

/// Result type alias for gopher-orch operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for gopher-orch operations.
#[derive(Debug)]
pub enum Error {
    /// Error loading or using the native library.
    Library(String),

    /// Error creating an agent.
    Agent(String),

    /// Invalid API key error.
    ApiKey(String),

    /// Connection error.
    Connection(String),

    /// Timeout error.
    Timeout(String),

    /// Configuration error.
    Config(String),

    /// Agent has been disposed.
    Disposed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Library(msg) => write!(f, "Library error: {}", msg),
            Error::Agent(msg) => write!(f, "Agent error: {}", msg),
            Error::ApiKey(msg) => write!(f, "API key error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Disposed => write!(f, "Agent has been disposed"),
        }
    }
}

impl StdError for Error {}

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

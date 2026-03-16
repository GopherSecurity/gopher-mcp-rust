//! Result types for agent queries.

use std::fmt;

/// Status of an agent result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentResultStatus {
    /// The query was successful.
    Success,
    /// An error occurred.
    Error,
    /// The query timed out.
    Timeout,
    /// Maximum iterations were reached.
    MaxIterationsReached,
}

impl fmt::Display for AgentResultStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentResultStatus::Success => write!(f, "SUCCESS"),
            AgentResultStatus::Error => write!(f, "ERROR"),
            AgentResultStatus::Timeout => write!(f, "TIMEOUT"),
            AgentResultStatus::MaxIterationsReached => write!(f, "MAX_ITERATIONS_REACHED"),
        }
    }
}

/// Result of an agent query with detailed information.
#[derive(Debug, Clone)]
pub struct AgentResult {
    response: String,
    status: AgentResultStatus,
    error_message: Option<String>,
    iteration_count: usize,
    tokens_used: u64,
}

impl AgentResult {
    /// Get the agent's response.
    pub fn response(&self) -> &str {
        &self.response
    }

    /// Get the result status.
    pub fn status(&self) -> AgentResultStatus {
        self.status
    }

    /// Get the error message (if any).
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Get the iteration count.
    pub fn iteration_count(&self) -> usize {
        self.iteration_count
    }

    /// Get the number of tokens used.
    pub fn tokens_used(&self) -> u64 {
        self.tokens_used
    }

    /// Check if the result was successful.
    pub fn is_success(&self) -> bool {
        self.status == AgentResultStatus::Success
    }

    /// Create a successful result.
    pub fn success<S: Into<String>>(response: S) -> Self {
        AgentResult {
            response: response.into(),
            status: AgentResultStatus::Success,
            error_message: None,
            iteration_count: 1,
            tokens_used: 0,
        }
    }

    /// Create an error result.
    pub fn error<S: Into<String>>(message: S) -> Self {
        AgentResult {
            response: String::new(),
            status: AgentResultStatus::Error,
            error_message: Some(message.into()),
            iteration_count: 0,
            tokens_used: 0,
        }
    }

    /// Create a timeout result.
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        AgentResult {
            response: String::new(),
            status: AgentResultStatus::Timeout,
            error_message: Some(message.into()),
            iteration_count: 0,
            tokens_used: 0,
        }
    }
}

/// Builder for creating AgentResult instances.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AgentResultBuilder {
    response: String,
    status: Option<AgentResultStatus>,
    error_message: Option<String>,
    iteration_count: usize,
    tokens_used: u64,
}

#[allow(dead_code)]
impl AgentResultBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the response.
    pub fn with_response<S: Into<String>>(mut self, response: S) -> Self {
        self.response = response.into();
        self
    }

    /// Set the status.
    pub fn with_status(mut self, status: AgentResultStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the error message.
    pub fn with_error_message<S: Into<String>>(mut self, message: S) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Set the iteration count.
    pub fn with_iteration_count(mut self, count: usize) -> Self {
        self.iteration_count = count;
        self
    }

    /// Set the tokens used.
    pub fn with_tokens_used(mut self, tokens: u64) -> Self {
        self.tokens_used = tokens;
        self
    }

    /// Build the AgentResult.
    pub fn build(self) -> AgentResult {
        AgentResult {
            response: self.response,
            status: self.status.unwrap_or(AgentResultStatus::Success),
            error_message: self.error_message,
            iteration_count: self.iteration_count,
            tokens_used: self.tokens_used,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let result = AgentResult::success("Hello, world!");

        assert_eq!(result.response(), "Hello, world!");
        assert_eq!(result.status(), AgentResultStatus::Success);
        assert!(result.is_success());
    }

    #[test]
    fn test_error_result() {
        let result = AgentResult::error("Something went wrong");

        assert_eq!(result.error_message(), Some("Something went wrong"));
        assert_eq!(result.status(), AgentResultStatus::Error);
        assert!(!result.is_success());
    }

    #[test]
    fn test_timeout_result() {
        let result = AgentResult::timeout("Operation timed out");

        assert_eq!(result.error_message(), Some("Operation timed out"));
        assert_eq!(result.status(), AgentResultStatus::Timeout);
        assert!(!result.is_success());
    }

    #[test]
    fn test_builder() {
        let result = AgentResultBuilder::new()
            .with_response("Test response")
            .with_status(AgentResultStatus::Success)
            .with_iteration_count(5)
            .with_tokens_used(100)
            .build();

        assert_eq!(result.response(), "Test response");
        assert_eq!(result.status(), AgentResultStatus::Success);
        assert_eq!(result.iteration_count(), 5);
        assert_eq!(result.tokens_used(), 100);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(AgentResultStatus::Success.to_string(), "SUCCESS");
        assert_eq!(AgentResultStatus::Error.to_string(), "ERROR");
        assert_eq!(AgentResultStatus::Timeout.to_string(), "TIMEOUT");
        assert_eq!(
            AgentResultStatus::MaxIterationsReached.to_string(),
            "MAX_ITERATIONS_REACHED"
        );
    }
}

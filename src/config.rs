//! Configuration types for creating agents.

/// Configuration for creating a GopherAgent.
#[derive(Debug, Clone, Default)]
pub struct Config {
    provider: String,
    model: String,
    api_key: Option<String>,
    server_config: Option<String>,
}

impl Config {
    /// Get the provider name.
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Get the model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the API key (if set).
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Get the server config JSON (if set).
    pub fn server_config(&self) -> Option<&str> {
        self.server_config.as_deref()
    }

    /// Check if an API key is configured.
    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().map_or(false, |k| !k.is_empty())
    }

    /// Check if a server config is configured.
    pub fn has_server_config(&self) -> bool {
        self.server_config.as_ref().map_or(false, |c| !c.is_empty())
    }
}

/// Builder for creating Config instances.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the provider name.
    pub fn with_provider<S: Into<String>>(mut self, provider: S) -> Self {
        self.config.provider = provider.into();
        self
    }

    /// Set the model name.
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the API key.
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.config.api_key = Some(api_key.into());
        self
    }

    /// Set the server config JSON.
    pub fn with_server_config<S: Into<String>>(mut self, server_config: S) -> Self {
        self.config.server_config = Some(server_config.into());
        self
    }

    /// Build the Config.
    pub fn build(self) -> Config {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_with_api_key() {
        let config = ConfigBuilder::new()
            .with_provider("AnthropicProvider")
            .with_model("claude-3-haiku-20240307")
            .with_api_key("test-api-key")
            .build();

        assert_eq!(config.provider(), "AnthropicProvider");
        assert_eq!(config.model(), "claude-3-haiku-20240307");
        assert!(config.has_api_key());
        assert!(!config.has_server_config());
        assert_eq!(config.api_key(), Some("test-api-key"));
    }

    #[test]
    fn test_builder_with_server_config() {
        let server_config = r#"{"succeeded": true}"#;
        let config = ConfigBuilder::new()
            .with_provider("AnthropicProvider")
            .with_model("claude-3-haiku-20240307")
            .with_server_config(server_config)
            .build();

        assert_eq!(config.provider(), "AnthropicProvider");
        assert_eq!(config.model(), "claude-3-haiku-20240307");
        assert!(!config.has_api_key());
        assert!(config.has_server_config());
        assert_eq!(config.server_config(), Some(server_config));
    }

    #[test]
    fn test_empty_config() {
        let config = ConfigBuilder::new().build();

        assert!(!config.has_api_key());
        assert!(!config.has_server_config());
        assert!(config.provider().is_empty());
        assert!(config.model().is_empty());
    }
}

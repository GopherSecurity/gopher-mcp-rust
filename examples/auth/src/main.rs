//! Rust Auth MCP Server
//!
//! An OAuth-protected MCP server example demonstrating JWT token validation
//! and scope-based access control for MCP tools.

mod config;
mod cors;
mod error;
mod ffi;
mod middleware;
mod routes;
mod tools;

use std::env;
use std::sync::Arc;

use axum::{
    extract::FromRef,
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AuthServerConfig;
use crate::ffi::GopherAuthClient;
use crate::middleware::{auth_middleware, AuthState};
use crate::routes::health::{health_handler, HealthState};
use crate::routes::mcp_handler::{mcp_handler, mcp_options, McpHandler};
use crate::routes::oauth_endpoints::{
    authorization_server_metadata, oauth_authorize, oauth_register, openid_configuration,
    protected_resource_metadata,
};
use crate::tools::weather_tools::register_weather_tools;

/// Combined application state.
#[derive(Clone)]
pub struct AppState {
    /// Server configuration.
    pub config: Arc<AuthServerConfig>,
    /// Health endpoint state.
    pub health: Arc<HealthState>,
    /// MCP handler state.
    pub mcp: Arc<McpHandler>,
}

// Implement FromRef to extract individual states
impl FromRef<AppState> for Arc<AuthServerConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Arc<HealthState> {
    fn from_ref(state: &AppState) -> Self {
        state.health.clone()
    }
}

impl FromRef<AppState> for Arc<McpHandler> {
    fn from_ref(state: &AppState) -> Self {
        state.mcp.clone()
    }
}

/// Print server banner.
fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                  Rust Auth MCP Server                    ║");
    println!("║           OAuth-Protected MCP Server Example             ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
}

/// Print available endpoints.
fn print_endpoints(config: &AuthServerConfig) {
    println!("Available Endpoints:");
    println!("────────────────────────────────────────────────────────────");
    println!("  Health:");
    println!("    GET  {}/health", config.server_url);
    println!();
    println!("  OAuth Discovery:");
    println!("    GET  {}/.well-known/oauth-protected-resource", config.server_url);
    println!("    GET  {}/.well-known/oauth-protected-resource/mcp", config.server_url);
    println!("    GET  {}/.well-known/oauth-authorization-server", config.server_url);
    println!("    GET  {}/.well-known/openid-configuration", config.server_url);
    println!();
    println!("  OAuth:");
    println!("    GET  {}/oauth/authorize", config.server_url);
    println!("    POST {}/oauth/register", config.server_url);
    println!();
    println!("  MCP (Protected):");
    println!("    POST {}/mcp", config.server_url);
    println!("    POST {}/rpc", config.server_url);
    println!("────────────────────────────────────────────────────────────");
    println!();
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    print_banner();

    // Load configuration
    let config_path = env::args().nth(1).unwrap_or_else(|| "server.config".to_string());

    let config = match AuthServerConfig::from_file(&config_path) {
        Ok(c) => {
            tracing::info!("Configuration loaded from {}", config_path);
            c
        }
        Err(e) => {
            tracing::warn!("Failed to load config from {}: {}", config_path, e);
            tracing::info!("Using default configuration with auth disabled");
            AuthServerConfig::default_disabled()
        }
    };

    // Initialize auth client if auth is enabled
    let auth_client: Option<Arc<GopherAuthClient>> = if config.auth_disabled {
        tracing::info!("Authentication is DISABLED");
        None
    } else {
        tracing::info!("Initializing auth library...");
        match GopherAuthClient::new(&config.jwks_uri, &config.issuer) {
            Ok(mut client) => {
                // Set client options
                if let Err(e) = client.set_option(
                    "cache_duration",
                    &config.jwks_cache_duration.to_string(),
                ) {
                    tracing::warn!("Failed to set cache_duration: {}", e);
                }
                if let Err(e) = client.set_option(
                    "auto_refresh",
                    if config.jwks_auto_refresh { "true" } else { "false" },
                ) {
                    tracing::warn!("Failed to set auto_refresh: {}", e);
                }
                if let Err(e) = client.set_option(
                    "request_timeout",
                    &config.request_timeout.to_string(),
                ) {
                    tracing::warn!("Failed to set request_timeout: {}", e);
                }
                tracing::info!("Auth library initialized successfully");
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize auth library: {}", e);
                tracing::warn!("Continuing with auth disabled");
                None
            }
        }
    };

    // Create shared state
    let config = Arc::new(config);
    let health_state = Arc::new(HealthState::new(Some("1.0.0".to_string())));
    let auth_state = Arc::new(AuthState::new(auth_client, (*config).clone()));

    // Create MCP handler and register tools
    let mut mcp = McpHandler::new();
    register_weather_tools(&mut mcp, config.auth_disabled);
    let mcp_state = Arc::new(mcp);

    // Create combined app state
    let app_state = AppState {
        config: config.clone(),
        health: health_state,
        mcp: mcp_state,
    };

    // Build router
    let app = Router::new()
        // Health endpoint
        .route("/health", get(health_handler))
        // OAuth discovery endpoints
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata).options(options_handler),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            get(protected_resource_metadata).options(options_handler),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata).options(options_handler),
        )
        .route(
            "/.well-known/openid-configuration",
            get(openid_configuration).options(options_handler),
        )
        // OAuth endpoints
        .route(
            "/oauth/authorize",
            get(oauth_authorize).options(options_handler),
        )
        .route(
            "/oauth/register",
            post(oauth_register).options(options_handler),
        )
        // MCP endpoints
        .route("/mcp", post(mcp_handler).options(mcp_options))
        .route("/rpc", post(mcp_handler).options(mcp_options))
        // Add combined state
        .with_state(app_state)
        // Add auth middleware
        .layer(axum_middleware::from_fn_with_state(
            auth_state,
            auth_middleware,
        ));

    // Print startup information
    let addr = format!("{}:{}", config.host, config.port);
    print_endpoints(&config);

    if config.auth_disabled {
        println!("⚠️  Authentication is DISABLED - all requests are allowed");
    } else {
        println!("🔒 Authentication is ENABLED");
        println!("   JWKS URI: {}", config.jwks_uri);
        println!("   Issuer: {}", config.issuer);
    }
    println!();

    tracing::info!("Server starting on {}", addr);
    println!("🚀 Server listening on http://{}", addr);
    println!();
    println!("Press Ctrl+C to shutdown");
    println!();

    // Start server
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind address");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Generic OPTIONS handler for CORS preflight.
async fn options_handler() -> impl axum::response::IntoResponse {
    cors::options_handler().await
}

//! Example using JSON server configuration.

use gopher_orch::{ConfigBuilder, GopherAgent};
use std::env;

/// Server configuration for local MCP servers
const SERVER_CONFIG: &str = r#"{
    "succeeded": true,
    "code": 200000000,
    "message": "success",
    "data": {
        "servers": [
            {
                "version": "2025-01-09",
                "serverId": "1",
                "name": "server1",
                "transport": "http_sse",
                "config": {"url": "http://127.0.0.1:3001/mcp", "headers": {}},
                "connectTimeout": 5000,
                "requestTimeout": 30000
            },
            {
                "version": "2025-01-09",
                "serverId": "2",
                "name": "server2",
                "transport": "http_sse",
                "config": {"url": "http://127.0.0.1:3002/mcp", "headers": {}},
                "connectTimeout": 5000,
                "requestTimeout": 30000
            }
        ]
    }
}"#;

fn main() {
    let provider = "AnthropicProvider";
    let model = "claude-3-haiku-20240307";

    // Create agent with JSON server configuration
    let config = ConfigBuilder::new()
        .with_provider(provider)
        .with_model(model)
        .with_server_config(SERVER_CONFIG)
        .build();

    let agent = match GopherAgent::create(config) {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("GopherAgent created!");

    // Get question from command line args or use default
    let args: Vec<String> = env::args().collect();
    let question = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "What is the weather like in New York?".to_string()
    };

    println!("Question: {}", question);

    // Run the query
    match agent.run(&question) {
        Ok(answer) => {
            println!("Answer:");
            println!("{}", answer);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

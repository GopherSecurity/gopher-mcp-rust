//! Rust Auth MCP Server
//!
//! An OAuth-protected MCP server example demonstrating JWT token validation
//! and scope-based access control for MCP tools.

mod error;
mod ffi;
mod middleware;
mod routes;
mod tools;

fn main() {
    println!("Rust Auth MCP Server");
}

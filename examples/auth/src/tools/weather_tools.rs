//! Weather Tools
//!
//! Example MCP tools demonstrating OAuth scope-based access control.
//! Mirrors the weather tools from the TypeScript and C++ auth examples.

use serde_json::{json, Value};

use crate::routes::mcp_handler::{AuthContext, McpHandler, ToolContent, ToolResult, ToolSpec};

/// Weather conditions for simulation.
const CONDITIONS: [&str; 6] = [
    "Sunny",
    "Cloudy",
    "Rainy",
    "Partly Cloudy",
    "Windy",
    "Stormy",
];

/// Day names for forecast.
const DAYS: [&str; 5] = ["Today", "Tomorrow", "Day 3", "Day 4", "Day 5"];

/// Check if a scope is present in a space-separated scope string.
///
/// # Arguments
///
/// * `scopes` - Space-separated scope string
/// * `required` - Required scope to check for
///
/// # Returns
///
/// true if the required scope is present
pub fn has_scope(scopes: &str, required: &str) -> bool {
    if scopes.is_empty() || required.is_empty() {
        return false;
    }
    scopes.split_whitespace().any(|s| s == required)
}

/// Calculate a simple hash of a city name.
///
/// # Arguments
///
/// * `city` - City name
///
/// # Returns
///
/// Sum of character byte values
pub fn city_hash(city: &str) -> usize {
    city.bytes().map(|b| b as usize).sum()
}

/// Get a deterministic but varying condition based on city name.
///
/// # Arguments
///
/// * `city` - City name
/// * `offset` - Offset for variation (e.g., day number)
///
/// # Returns
///
/// Weather condition string
pub fn get_condition(city: &str, offset: usize) -> &'static str {
    let hash = city_hash(city);
    CONDITIONS[(hash + offset) % CONDITIONS.len()]
}

/// Get a deterministic but varying temperature based on city name.
///
/// # Arguments
///
/// * `city` - City name
/// * `offset` - Offset for variation (e.g., day number)
///
/// # Returns
///
/// Temperature in Celsius (10-35°C range)
pub fn get_temp(city: &str, offset: usize) -> i32 {
    let hash = city_hash(city);
    10 + ((hash + offset * 7) % 26) as i32
}

/// Create an access denied error result.
///
/// # Arguments
///
/// * `scope` - Required scope that was missing
///
/// # Returns
///
/// ToolResult with error
pub fn access_denied(scope: &str) -> ToolResult {
    ToolResult {
        content: vec![ToolContent::text(
            serde_json::to_string(&json!({
                "error": "access_denied",
                "message": format!("Access denied. Required scope: {}", scope),
            }))
            .unwrap_or_else(|_| "{}".to_string()),
        )],
        is_error: Some(true),
    }
}

/// Get simulated current weather for a city.
fn get_simulated_weather(city: &str) -> Value {
    let hash = city_hash(city);

    json!({
        "city": city,
        "temperature": get_temp(city, 0),
        "condition": get_condition(city, 0),
        "humidity": 40 + (hash % 40),  // 40-80%
        "windSpeed": 5 + (hash % 25),  // 5-30 km/h
    })
}

/// Get simulated 5-day forecast for a city.
fn get_simulated_forecast(city: &str) -> Value {
    let forecast: Vec<Value> = DAYS
        .iter()
        .enumerate()
        .map(|(index, day)| {
            json!({
                "day": day,
                "high": get_temp(city, index) + 5,
                "low": get_temp(city, index) - 5,
                "condition": get_condition(city, index),
            })
        })
        .collect();

    json!({
        "city": city,
        "forecast": forecast,
    })
}

/// Get simulated weather alerts for a region.
fn get_simulated_alerts(region: &str) -> Value {
    let hash = city_hash(region);

    let alerts: Vec<Value> = if hash % 3 == 0 {
        vec![json!({
            "type": "Heat Warning",
            "severity": "moderate",
            "message": format!("High temperatures expected in {}. Stay hydrated.", region),
        })]
    } else if hash % 3 == 1 {
        vec![
            json!({
                "type": "Storm Watch",
                "severity": "high",
                "message": format!("Severe thunderstorms possible in {}. Seek shelter if needed.", region),
            }),
            json!({
                "type": "Wind Advisory",
                "severity": "low",
                "message": format!("Strong winds expected in {}. Secure loose objects.", region),
            }),
        ]
    } else {
        vec![] // No alerts
    };

    json!({
        "region": region,
        "alerts": alerts,
    })
}

/// Register weather tools with the MCP handler.
///
/// # Arguments
///
/// * `mcp` - MCP handler instance
/// * `auth_disabled` - Whether authentication is disabled
pub fn register_weather_tools(mcp: &mut McpHandler, auth_disabled: bool) {
    // get-weather - No authentication required
    // Returns current weather for a specified city.
    mcp.register_tool(
        "get-weather",
        ToolSpec {
            name: "get-weather".to_string(),
            description: "Get current weather for a city. No authentication required.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name to get weather for"
                    }
                },
                "required": ["city"]
            }),
        },
        |args, _auth_context| {
            let city = args
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let weather = get_simulated_weather(city);
            ToolResult::text(serde_json::to_string_pretty(&weather).unwrap_or_default())
        },
    );

    // get-forecast - Requires mcp:read scope
    // Returns 5-day weather forecast for a specified city.
    let auth_disabled_forecast = auth_disabled;
    mcp.register_tool(
        "get-forecast",
        ToolSpec {
            name: "get-forecast".to_string(),
            description: "Get 5-day weather forecast for a city. Requires mcp:read scope."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name to get forecast for"
                    }
                },
                "required": ["city"]
            }),
        },
        move |args, auth_context| {
            // Check scope if auth is enabled
            if !auth_disabled_forecast && !has_scope(&auth_context.scopes, "mcp:read") {
                return access_denied("mcp:read");
            }

            let city = args
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let forecast = get_simulated_forecast(city);
            ToolResult::text(serde_json::to_string_pretty(&forecast).unwrap_or_default())
        },
    );

    // get-weather-alerts - Requires mcp:admin scope
    // Returns weather alerts for a specified region.
    let auth_disabled_alerts = auth_disabled;
    mcp.register_tool(
        "get-weather-alerts",
        ToolSpec {
            name: "get-weather-alerts".to_string(),
            description: "Get weather alerts for a region. Requires mcp:admin scope.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "region": {
                        "type": "string",
                        "description": "Region name to get alerts for"
                    }
                },
                "required": ["region"]
            }),
        },
        move |args, auth_context| {
            // Check scope if auth is enabled
            if !auth_disabled_alerts && !has_scope(&auth_context.scopes, "mcp:admin") {
                return access_denied("mcp:admin");
            }

            let region = args
                .get("region")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let alerts = get_simulated_alerts(region);
            ToolResult::text(serde_json::to_string_pretty(&alerts).unwrap_or_default())
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_scope_present() {
        assert!(has_scope("openid profile mcp:read", "mcp:read"));
        assert!(has_scope("openid profile mcp:read", "openid"));
        assert!(has_scope("openid profile mcp:read", "profile"));
    }

    #[test]
    fn test_has_scope_absent() {
        assert!(!has_scope("openid profile", "mcp:read"));
        assert!(!has_scope("openid profile mcp:read", "mcp:admin"));
    }

    #[test]
    fn test_has_scope_empty() {
        assert!(!has_scope("", "mcp:read"));
        assert!(!has_scope("openid profile", ""));
        assert!(!has_scope("", ""));
    }

    #[test]
    fn test_city_hash() {
        // "NYC" = 78 + 89 + 67 = 234
        assert_eq!(city_hash("NYC"), 234);
        // Hash should be consistent
        assert_eq!(city_hash("NYC"), city_hash("NYC"));
        // Different cities have different hashes
        assert_ne!(city_hash("NYC"), city_hash("LA"));
    }

    #[test]
    fn test_get_condition_deterministic() {
        let condition1 = get_condition("NYC", 0);
        let condition2 = get_condition("NYC", 0);
        assert_eq!(condition1, condition2);
    }

    #[test]
    fn test_get_condition_varies_with_offset() {
        // Different offsets should give different conditions (for most cities)
        let conditions: Vec<&str> = (0..6).map(|i| get_condition("NYC", i)).collect();
        // At least some should be different
        let unique_count = conditions
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(unique_count > 1);
    }

    #[test]
    fn test_get_temp_range() {
        // Temperature should be in 10-35 range
        for city in &["NYC", "LA", "Chicago", "Seattle", "Miami"] {
            let temp = get_temp(city, 0);
            assert!(temp >= 10 && temp <= 35, "Temp for {} was {}", city, temp);
        }
    }

    #[test]
    fn test_get_temp_deterministic() {
        assert_eq!(get_temp("NYC", 0), get_temp("NYC", 0));
    }

    #[test]
    fn test_access_denied_result() {
        let result = access_denied("mcp:read");
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);

        let text = result.content[0].text.as_ref().unwrap();
        assert!(text.contains("access_denied"));
        assert!(text.contains("mcp:read"));
    }

    #[test]
    fn test_simulated_weather() {
        let weather = get_simulated_weather("NYC");
        assert_eq!(weather["city"], "NYC");
        assert!(weather["temperature"].is_number());
        assert!(weather["condition"].is_string());
        assert!(weather["humidity"].is_number());
        assert!(weather["windSpeed"].is_number());
    }

    #[test]
    fn test_simulated_forecast() {
        let forecast = get_simulated_forecast("NYC");
        assert_eq!(forecast["city"], "NYC");
        let days = forecast["forecast"].as_array().unwrap();
        assert_eq!(days.len(), 5);

        for day in days {
            assert!(day["day"].is_string());
            assert!(day["high"].is_number());
            assert!(day["low"].is_number());
            assert!(day["condition"].is_string());
        }
    }

    #[test]
    fn test_simulated_alerts() {
        // Test different regions to exercise all branches
        let alerts1 = get_simulated_alerts("Region1");
        let alerts2 = get_simulated_alerts("Region2");
        let alerts3 = get_simulated_alerts("Region3");

        // All should have region and alerts fields
        assert!(alerts1.get("region").is_some());
        assert!(alerts1.get("alerts").is_some());
        assert!(alerts2.get("region").is_some());
        assert!(alerts2.get("alerts").is_some());
        assert!(alerts3.get("region").is_some());
        assert!(alerts3.get("alerts").is_some());
    }

    #[test]
    fn test_register_weather_tools() {
        let mut mcp = McpHandler::new();
        register_weather_tools(&mut mcp, true);

        // Test get-weather tool
        let auth_context = AuthContext::default();
        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get-weather",
                    "arguments": {"city": "NYC"}
                }
            }),
            &auth_context,
        );

        assert!(result.result.is_some());
    }

    #[test]
    fn test_get_forecast_with_scope() {
        let mut mcp = McpHandler::new();
        register_weather_tools(&mut mcp, false);

        // Without proper scope should fail
        let auth_context = AuthContext {
            user_id: "user1".to_string(),
            scopes: "openid".to_string(),
            ..Default::default()
        };

        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get-forecast",
                    "arguments": {"city": "NYC"}
                }
            }),
            &auth_context,
        );

        // Result should contain access_denied error
        let result_value = result.result.unwrap();
        assert_eq!(result_value["isError"], true);

        // With proper scope should succeed
        let auth_context = AuthContext {
            user_id: "user1".to_string(),
            scopes: "openid mcp:read".to_string(),
            ..Default::default()
        };

        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get-forecast",
                    "arguments": {"city": "NYC"}
                }
            }),
            &auth_context,
        );

        let result_value = result.result.unwrap();
        assert!(result_value.get("isError").is_none());
    }

    #[test]
    fn test_get_weather_alerts_with_scope() {
        let mut mcp = McpHandler::new();
        register_weather_tools(&mut mcp, false);

        // Without proper scope should fail
        let auth_context = AuthContext {
            user_id: "user1".to_string(),
            scopes: "openid mcp:read".to_string(),
            ..Default::default()
        };

        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get-weather-alerts",
                    "arguments": {"region": "Northeast"}
                }
            }),
            &auth_context,
        );

        // Result should contain access_denied error
        let result_value = result.result.unwrap();
        assert_eq!(result_value["isError"], true);

        // With proper scope should succeed
        let auth_context = AuthContext {
            user_id: "user1".to_string(),
            scopes: "openid mcp:admin".to_string(),
            ..Default::default()
        };

        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get-weather-alerts",
                    "arguments": {"region": "Northeast"}
                }
            }),
            &auth_context,
        );

        let result_value = result.result.unwrap();
        assert!(result_value.get("isError").is_none());
    }

    #[test]
    fn test_tools_list() {
        let mut mcp = McpHandler::new();
        register_weather_tools(&mut mcp, true);

        let auth_context = AuthContext::default();
        let result = mcp.handle_request(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
            &auth_context,
        );

        let result_value = result.result.unwrap();
        let tools = result_value["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(tool_names.contains(&"get-weather"));
        assert!(tool_names.contains(&"get-forecast"));
        assert!(tool_names.contains(&"get-weather-alerts"));
    }
}

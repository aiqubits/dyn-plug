use dyn_plug_core::{Plugin, register_plugin};
use std::error::Error;

/// Plugin A - String Processing Plugin
/// 
/// This plugin provides string processing operations including:
/// - uppercase: Convert string to uppercase
/// - lowercase: Convert string to lowercase  
/// - reverse: Reverse the string
/// 
/// Input format: JSON with "operation" and "text" fields
/// Example: {"operation": "uppercase", "text": "hello world"}
pub struct PluginA;

impl PluginA {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for PluginA {
    fn name(&self) -> &str {
        "plugin_a"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "String processing plugin with uppercase, lowercase, and reverse operations"
    }

    fn execute(&self, input: &str) -> Result<String, Box<dyn Error>> {
        // Parse JSON input
        let parsed: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| format!("Invalid JSON input: {}", e))?;

        let operation = parsed["operation"]
            .as_str()
            .ok_or("Missing 'operation' field")?;

        let text = parsed["text"]
            .as_str()
            .ok_or("Missing 'text' field")?;

        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            _ => return Err(format!("Unknown operation: {}. Supported operations: uppercase, lowercase, reverse", operation).into()),
        };

        // Return result as JSON
        let response = serde_json::json!({
            "operation": operation,
            "input": text,
            "output": result
        });

        Ok(response.to_string())
    }
}

register_plugin!(PluginA);
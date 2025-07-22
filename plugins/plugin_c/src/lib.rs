use dyn_plug_core::{Plugin, register_plugin};
use std::error::Error;

/// Plugin C - JSON Processing Plugin
/// 
/// This plugin provides JSON processing operations including:
/// - format: Pretty-format JSON with indentation
/// - minify: Minify JSON by removing whitespace
/// - validate: Validate JSON syntax
/// - query: Extract value from JSON using dot notation (e.g., "user.name")
/// - keys: Get all keys from a JSON object
/// - type: Get the type of a JSON value
/// 
/// Input format: JSON with "operation" and "data" fields
/// Example: {"operation": "format", "data": "{\"name\":\"John\",\"age\":30}"}
/// For query: {"operation": "query", "data": "{\"user\":{\"name\":\"John\"}}", "path": "user.name"}
pub struct PluginC;

impl PluginC {
    pub fn new() -> Self {
        Self
    }

    fn query_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;
        
        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(part)?;
                },
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                },
                _ => return None,
            }
        }
        
        Some(current)
    }

    fn get_json_keys(value: &serde_json::Value) -> Vec<String> {
        match value {
            serde_json::Value::Object(map) => map.keys().cloned().collect(),
            _ => vec![],
        }
    }

    fn get_json_type(value: &serde_json::Value) -> &'static str {
        match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
}

impl Plugin for PluginC {
    fn name(&self) -> &str {
        "plugin_c"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "JSON processing plugin with format, minify, validate, query, keys, and type operations"
    }

    fn execute(&self, input: &str) -> Result<String, Box<dyn Error>> {
        // Parse JSON input
        let parsed: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| format!("Invalid JSON input: {}", e))?;

        let operation = parsed["operation"]
            .as_str()
            .ok_or("Missing 'operation' field")?;

        let data_str = parsed["data"]
            .as_str()
            .ok_or("Missing 'data' field")?;

        let result = match operation {
            "format" => {
                let json_data: serde_json::Value = serde_json::from_str(data_str)
                    .map_err(|e| format!("Invalid JSON data: {}", e))?;
                serde_json::to_string_pretty(&json_data)
                    .map_err(|e| format!("Failed to format JSON: {}", e))?
            },
            "minify" => {
                let json_data: serde_json::Value = serde_json::from_str(data_str)
                    .map_err(|e| format!("Invalid JSON data: {}", e))?;
                serde_json::to_string(&json_data)
                    .map_err(|e| format!("Failed to minify JSON: {}", e))?
            },
            "validate" => {
                match serde_json::from_str::<serde_json::Value>(data_str) {
                    Ok(_) => "Valid JSON".to_string(),
                    Err(e) => format!("Invalid JSON: {}", e),
                }
            },
            "query" => {
                let path = parsed["path"]
                    .as_str()
                    .ok_or("Missing 'path' field for query operation")?;
                
                let json_data: serde_json::Value = serde_json::from_str(data_str)
                    .map_err(|e| format!("Invalid JSON data: {}", e))?;
                
                match Self::query_json_path(&json_data, path) {
                    Some(value) => serde_json::to_string(value)
                        .map_err(|e| format!("Failed to serialize query result: {}", e))?,
                    None => "null".to_string(),
                }
            },
            "keys" => {
                let json_data: serde_json::Value = serde_json::from_str(data_str)
                    .map_err(|e| format!("Invalid JSON data: {}", e))?;
                
                let keys = Self::get_json_keys(&json_data);
                serde_json::to_string(&keys)
                    .map_err(|e| format!("Failed to serialize keys: {}", e))?
            },
            "type" => {
                let json_data: serde_json::Value = serde_json::from_str(data_str)
                    .map_err(|e| format!("Invalid JSON data: {}", e))?;
                
                Self::get_json_type(&json_data).to_string()
            },
            _ => return Err(format!("Unknown operation: {}. Supported operations: format, minify, validate, query, keys, type", operation).into()),
        };

        // Return result as JSON
        let response = serde_json::json!({
            "operation": operation,
            "input": data_str,
            "output": result
        });

        Ok(response.to_string())
    }
}

register_plugin!(PluginC);
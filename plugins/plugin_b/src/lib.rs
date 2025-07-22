use dyn_plug_core::{Plugin, register_plugin};
use std::error::Error;

/// Plugin B - Numeric Processing Plugin
/// 
/// This plugin provides numeric processing operations including:
/// - add: Add two numbers
/// - subtract: Subtract second number from first
/// - multiply: Multiply two numbers
/// - divide: Divide first number by second
/// - power: Raise first number to the power of second
/// - sqrt: Square root of a number
/// 
/// Input format: JSON with "operation" and "numbers" fields
/// Example: {"operation": "add", "numbers": [5, 3]}
/// For single number operations: {"operation": "sqrt", "numbers": [16]}
pub struct PluginB;

impl PluginB {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for PluginB {
    fn name(&self) -> &str {
        "plugin_b"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Numeric processing plugin with arithmetic operations (add, subtract, multiply, divide, power, sqrt)"
    }

    fn execute(&self, input: &str) -> Result<String, Box<dyn Error>> {
        // Parse JSON input
        let parsed: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| format!("Invalid JSON input: {}", e))?;

        let operation = parsed["operation"]
            .as_str()
            .ok_or("Missing 'operation' field")?;

        let numbers = parsed["numbers"]
            .as_array()
            .ok_or("Missing 'numbers' field or not an array")?;

        let result = match operation {
            "add" => {
                if numbers.len() != 2 {
                    return Err("Add operation requires exactly 2 numbers".into());
                }
                let a = numbers[0].as_f64().ok_or("First number is not a valid number")?;
                let b = numbers[1].as_f64().ok_or("Second number is not a valid number")?;
                a + b
            },
            "subtract" => {
                if numbers.len() != 2 {
                    return Err("Subtract operation requires exactly 2 numbers".into());
                }
                let a = numbers[0].as_f64().ok_or("First number is not a valid number")?;
                let b = numbers[1].as_f64().ok_or("Second number is not a valid number")?;
                a - b
            },
            "multiply" => {
                if numbers.len() != 2 {
                    return Err("Multiply operation requires exactly 2 numbers".into());
                }
                let a = numbers[0].as_f64().ok_or("First number is not a valid number")?;
                let b = numbers[1].as_f64().ok_or("Second number is not a valid number")?;
                a * b
            },
            "divide" => {
                if numbers.len() != 2 {
                    return Err("Divide operation requires exactly 2 numbers".into());
                }
                let a = numbers[0].as_f64().ok_or("First number is not a valid number")?;
                let b = numbers[1].as_f64().ok_or("Second number is not a valid number")?;
                if b == 0.0 {
                    return Err("Division by zero is not allowed".into());
                }
                a / b
            },
            "power" => {
                if numbers.len() != 2 {
                    return Err("Power operation requires exactly 2 numbers".into());
                }
                let a = numbers[0].as_f64().ok_or("First number is not a valid number")?;
                let b = numbers[1].as_f64().ok_or("Second number is not a valid number")?;
                a.powf(b)
            },
            "sqrt" => {
                if numbers.len() != 1 {
                    return Err("Square root operation requires exactly 1 number".into());
                }
                let a = numbers[0].as_f64().ok_or("Number is not a valid number")?;
                if a < 0.0 {
                    return Err("Cannot calculate square root of negative number".into());
                }
                a.sqrt()
            },
            _ => return Err(format!("Unknown operation: {}. Supported operations: add, subtract, multiply, divide, power, sqrt", operation).into()),
        };

        // Return result as JSON
        let response = serde_json::json!({
            "operation": operation,
            "input": numbers,
            "output": result
        });

        Ok(response.to_string())
    }
}

register_plugin!(PluginB);
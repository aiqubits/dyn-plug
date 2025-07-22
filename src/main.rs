use clap::{Parser, Subcommand};
use dyn_plug_core::{PluginManager, PluginError};
use log::{error, info};
use std::process;

mod api;

#[derive(Parser)]
#[command(name = "dyn-plug")]
#[command(about = "A pluggable service system")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available plugins with their status
    List,
    /// Enable a plugin
    Enable {
        /// Name of the plugin to enable
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Name of the plugin to disable
        name: String,
    },
    /// Execute a plugin with optional input
    Execute {
        /// Name of the plugin to execute
        name: String,
        /// Input to pass to the plugin (optional)
        #[arg(short, long)]
        input: Option<String>,
    },
    /// Start the HTTP API server
    Serve {
        /// Port to bind the server to
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind the server to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

fn main() {
    // Initialize logging
    env_logger::init();
    
    let cli = Cli::parse();
    
    // Initialize plugin manager
    let mut manager = match PluginManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize plugin manager: {}", e);
            process::exit(1);
        }
    };
    
    // Execute the requested command
    let result = match cli.command {
        Commands::List => handle_list(&manager),
        Commands::Enable { name } => handle_enable(&mut manager, &name),
        Commands::Disable { name } => handle_disable(&mut manager, &name),
        Commands::Execute { name, input } => handle_execute(&manager, &name, input.as_deref()),
        Commands::Serve { port, host } => handle_serve(manager, &host, port),
    };
    
    // Handle command result
    if let Err(e) = result {
        error!("Command failed: {}", e);
        process::exit(1);
    }
}

fn handle_list(manager: &PluginManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listing all plugins");
    
    let plugins = manager.list_plugins();
    
    if plugins.is_empty() {
        println!("No plugins found.");
        return Ok(());
    }
    
    println!("Available plugins:");
    println!("{:<20} {:<10} {:<10} {:<50}", "Name", "Version", "Status", "Description");
    println!("{}", "-".repeat(90));
    
    for plugin in plugins {
        let status = if plugin.enabled && plugin.config_enabled {
            "enabled"
        } else {
            "disabled"
        };
        
        println!(
            "{:<20} {:<10} {:<10} {:<50}",
            plugin.name,
            plugin.version,
            status,
            truncate_string(&plugin.description, 50)
        );
    }
    
    Ok(())
}

fn handle_enable(manager: &mut PluginManager, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Enabling plugin: {}", name);
    
    match manager.enable_plugin(name) {
        Ok(()) => {
            println!("Plugin '{}' enabled successfully.", name);
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("Plugin '{}' not found.", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(e) => {
            error!("Failed to enable plugin '{}': {}", name, e);
            Err(format!("Failed to enable plugin '{}': {}", name, e).into())
        }
    }
}

fn handle_disable(manager: &mut PluginManager, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Disabling plugin: {}", name);
    
    match manager.disable_plugin(name) {
        Ok(()) => {
            println!("Plugin '{}' disabled successfully.", name);
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("Plugin '{}' not found.", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(e) => {
            error!("Failed to disable plugin '{}': {}", name, e);
            Err(format!("Failed to disable plugin '{}': {}", name, e).into())
        }
    }
}

fn handle_execute(
    manager: &PluginManager,
    name: &str,
    input: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_str = input.unwrap_or("");
    info!("Executing plugin '{}' with input: '{}'", name, input_str);
    
    match manager.execute_plugin(name, input_str) {
        Ok(result) => {
            if result.success {
                println!("Plugin '{}' executed successfully:", name);
                println!("Output: {}", result.output);
                println!("Duration: {}ms", result.duration_ms);
            } else {
                error!("Plugin '{}' execution failed: {}", name, result.output);
                return Err(format!("Plugin execution failed: {}", result.output).into());
            }
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("Plugin '{}' not found.", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(PluginError::PluginDisabled { .. }) => {
            error!("Plugin '{}' is disabled.", name);
            Err(format!("Plugin '{}' is disabled. Use 'enable {}' to enable it first.", name, name).into())
        }
        Err(e) => {
            error!("Failed to execute plugin '{}': {}", name, e);
            Err(format!("Failed to execute plugin '{}': {}", name, e).into())
        }
    }
}

fn handle_serve(
    manager: PluginManager,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTTP API server on {}:{}", host, port);
    
    let host_owned = host.to_string();
    
    // Create a new Tokio runtime for the server
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async move {
        // Set up signal handling for graceful shutdown
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
        
        // Handle Ctrl+C signal
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
            info!("Received shutdown signal");
            let _ = tx.send(()).await;
        });
        
        println!("HTTP API server starting on {}:{}", host_owned, port);
        println!("Available endpoints:");
        println!("  GET    /health                     - Health check");
        println!("  GET    /api/v1/plugins             - List all plugins");
        println!("  POST   /api/v1/plugins/{{name}}/execute - Execute plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/enable  - Enable plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/disable - Disable plugin");
        println!("Press Ctrl+C to stop the server");
        
        // Start the server directly without spawning
        tokio::select! {
            result = api::start_server(manager, &host_owned, port) => {
                if let Err(e) = result {
                    error!("Server error: {}", e);
                }
            }
            _ = rx.recv() => {
                info!("Shutdown signal received, stopping server...");
            }
        }
        
        println!("Shutting down server...");
        info!("Server shutdown complete");
        
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    
    Ok(())
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a very long string", 10), "this is...");
        assert_eq!(truncate_string("exactly10!", 10), "exactly10!");
        assert_eq!(truncate_string("", 5), "");
    }

    #[test]
    fn test_cli_parsing() {
        // Test that CLI can be parsed (basic smoke test)
        let cli = Cli::try_parse_from(&["dyn-plug", "list"]);
        assert!(cli.is_ok());
        
        let cli = Cli::try_parse_from(&["dyn-plug", "enable", "test-plugin"]);
        assert!(cli.is_ok());
        
        let cli = Cli::try_parse_from(&["dyn-plug", "execute", "test-plugin", "--input", "test"]);
        assert!(cli.is_ok());
    }
}
use clap::{Parser, Subcommand};
use dyn_plug_core::{PluginManager, PluginError};
use log::{error, info, warn};
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
        // Set up graceful shutdown handling
        let shutdown_manager = ShutdownManager::new();
        let shutdown_signal = shutdown_manager.setup_signal_handling().await?;
        
        println!("HTTP API server starting on {}:{}", host_owned, port);
        println!("Available endpoints:");
        println!("  GET    /health                     - Health check");
        println!("  GET    /api/v1/plugins             - List all plugins");
        println!("  POST   /api/v1/plugins/{{name}}/execute - Execute plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/enable  - Enable plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/disable - Disable plugin");
        println!("Press Ctrl+C to stop the server");
        
        // Start the server with graceful shutdown handling
        let server_result = run_server_with_shutdown(manager, &host_owned, port, shutdown_signal).await;
        
        // Perform cleanup
        shutdown_manager.cleanup().await;
        
        match server_result {
            Ok(()) => {
                println!("Server shutdown completed successfully");
                info!("Server shutdown complete");
            }
            Err(e) => {
                error!("Server encountered an error during shutdown: {}", e);
                return Err(e);
            }
        }
        
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    
    Ok(())
}

/// Run the server with graceful shutdown handling
async fn run_server_with_shutdown(
    manager: PluginManager,
    host: &str,
    port: u16,
    mut shutdown_signal: tokio::sync::mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start the server with graceful shutdown handling
    match api::start_server(manager, host, port, &mut shutdown_signal).await {
        Ok(()) => {
            info!("Server shut down gracefully");
            Ok(())
        }
        Err(e) => {
            // Log the error but handle network errors gracefully
            if is_recoverable_network_error(&e) {
                warn!("Recoverable network error occurred: {}", e);
                info!("Server stopped due to network error, but this is recoverable");
                Ok(())
            } else {
                error!("Server failed with non-recoverable error: {}", e);
                Err(e)
            }
        }
    }
}

/// Check if an error is a recoverable network error
fn is_recoverable_network_error(error: &Box<dyn std::error::Error>) -> bool {
    let error_str = error.to_string().to_lowercase();
    
    // Common recoverable network errors
    error_str.contains("address already in use") ||
    error_str.contains("connection refused") ||
    error_str.contains("network unreachable") ||
    error_str.contains("temporary failure")
}

/// Manages graceful shutdown of the service
struct ShutdownManager {
    cleanup_tasks: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl ShutdownManager {
    fn new() -> Self {
        Self {
            cleanup_tasks: Vec::new(),
        }
    }
    
    /// Set up signal handling for graceful shutdown
    async fn setup_signal_handling(&self) -> Result<tokio::sync::mpsc::Receiver<()>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
        
        // Use ctrlc crate for cross-platform signal handling
        let tx_clone = tx.clone();
        ctrlc::set_handler(move || {
            info!("Received shutdown signal (Ctrl+C)");
            if let Err(e) = tx_clone.blocking_send(()) {
                error!("Failed to send shutdown signal: {}", e);
            }
        })?;
        
        // Also handle SIGTERM on Unix systems
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())?;
            let tx_sigterm = tx.clone();
            
            tokio::spawn(async move {
                sigterm.recv().await;
                info!("Received SIGTERM signal");
                if let Err(e) = tx_sigterm.send(()).await {
                    error!("Failed to send SIGTERM shutdown signal: {}", e);
                }
            });
        }
        
        Ok(rx)
    }
    
    /// Perform cleanup tasks
    async fn cleanup(&self) {
        info!("Performing cleanup tasks...");
        
        for task in &self.cleanup_tasks {
            task();
        }
        
        // Give a moment for any async cleanup to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        info!("Cleanup completed");
    }
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
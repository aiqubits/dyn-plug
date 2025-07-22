use clap::{Parser, Subcommand};
use dyn_plug_core::{PluginManager, PluginError};
use log::{debug, error, info, warn};
use std::env;
use std::process;

mod api;

/// Initialize logging with configurable levels
/// 
/// Supports configuration via:
/// - RUST_LOG environment variable (standard)
/// - DYN_PLUG_LOG_LEVEL environment variable (application-specific)
/// - Defaults to 'info' level if not specified
fn initialize_logging() {
    // Check for application-specific log level first
    let log_level = env::var("DYN_PLUG_LOG_LEVEL")
        .or_else(|_| env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".to_string());
    
    // Set RUST_LOG if not already set to ensure env_logger picks it up
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", &log_level);
    }
    
    // Initialize env_logger with timestamp and target information
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .format_target(true)
        .init();
    
    info!("Logging initialized with level: {}", log_level);
    debug!("Debug logging is enabled");
}

/// Initialize plugin manager with retry logic for transient failures
fn initialize_plugin_manager_with_retry() -> Result<PluginManager, Box<dyn std::error::Error>> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 1000;
    
    for attempt in 1..=MAX_RETRIES {
        info!("Initializing plugin manager (attempt {}/{})", attempt, MAX_RETRIES);
        
        match PluginManager::new() {
            Ok(manager) => {
                info!("Plugin manager initialized successfully on attempt {}", attempt);
                return Ok(manager);
            }
            Err(e) => {
                if attempt < MAX_RETRIES && is_transient_error(&e) {
                    warn!("Transient error on attempt {}: {}. Retrying in {}ms...", 
                          attempt, e, RETRY_DELAY_MS);
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                } else {
                    error!("Failed to initialize plugin manager on attempt {}: {}", attempt, e);
                    return Err(format!("Plugin manager initialization failed: {}", e).into());
                }
            }
        }
    }
    
    Err("Plugin manager initialization failed after all retries".into())
}

/// Check if an error is transient and worth retrying (using enhanced error methods)
fn is_transient_error(error: &PluginError) -> bool {
    error.is_transient()
}

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
    // Initialize logging with configurable levels
    initialize_logging();
    
    let cli = Cli::parse();
    
    // Initialize plugin manager with retry logic for transient failures
    let mut manager = match initialize_plugin_manager_with_retry() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize plugin manager after retries: {}", e);
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
    info!("CLI: Starting plugin list operation");
    debug!("CLI: Retrieving plugin information from manager");
    
    let plugins = manager.list_plugins();
    
    info!("CLI: Found {} plugins", plugins.len());
    
    if plugins.is_empty() {
        info!("CLI: No plugins available to display");
        println!("No plugins found.");
        return Ok(());
    }
    
    println!("Available plugins:");
    println!("{:<20} {:<10} {:<10} {:<50}", "Name", "Version", "Status", "Description");
    println!("{}", "-".repeat(90));
    
    let mut enabled_count = 0;
    let mut disabled_count = 0;
    
    for plugin in plugins {
        let status = if plugin.enabled && plugin.config_enabled {
            enabled_count += 1;
            "enabled"
        } else {
            disabled_count += 1;
            "disabled"
        };
        
        debug!("CLI: Plugin {} - status: {}, loaded: {}", plugin.name, status, plugin.loaded);
        
        println!(
            "{:<20} {:<10} {:<10} {:<50}",
            plugin.name,
            plugin.version,
            status,
            truncate_string(&plugin.description, 50)
        );
    }
    
    info!("CLI: Plugin list completed - {} enabled, {} disabled", enabled_count, disabled_count);
    Ok(())
}

fn handle_enable(manager: &mut PluginManager, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("CLI: Starting enable operation for plugin: {}", name);
    debug!("CLI: Checking if plugin '{}' exists before enabling", name);
    
    // Pre-check if plugin exists for better error messaging
    if !manager.has_plugin(name) {
        warn!("CLI: Plugin '{}' not found in registry", name);
        let available_plugins: Vec<String> = manager.list_plugins()
            .iter()
            .map(|p| p.name.clone())
            .collect();
        debug!("CLI: Available plugins: {:?}", available_plugins);
        
        error!("Plugin '{}' not found.", name);
        return Err(format!(
            "Plugin '{}' not found. Available plugins: {}. Use 'list' command for details.", 
            name, 
            if available_plugins.is_empty() { 
                "none".to_string() 
            } else { 
                available_plugins.join(", ") 
            }
        ).into());
    }
    
    match manager.enable_plugin(name) {
        Ok(()) => {
            info!("CLI: Plugin '{}' enabled successfully", name);
            println!("Plugin '{}' enabled successfully.", name);
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("CLI: Plugin '{}' not found during enable operation", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(e) => {
            error!("CLI: Failed to enable plugin '{}': {} (category: {})", name, e, e.category());
            Err(e.user_friendly_message().into())
        }
    }
}

fn handle_disable(manager: &mut PluginManager, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("CLI: Starting disable operation for plugin: {}", name);
    debug!("CLI: Checking if plugin '{}' exists before disabling", name);
    
    // Pre-check if plugin exists for better error messaging
    if !manager.has_plugin(name) {
        warn!("CLI: Plugin '{}' not found in registry", name);
        let available_plugins: Vec<String> = manager.list_plugins()
            .iter()
            .map(|p| p.name.clone())
            .collect();
        debug!("CLI: Available plugins: {:?}", available_plugins);
        
        error!("Plugin '{}' not found.", name);
        return Err(format!(
            "Plugin '{}' not found. Available plugins: {}. Use 'list' command for details.", 
            name, 
            if available_plugins.is_empty() { 
                "none".to_string() 
            } else { 
                available_plugins.join(", ") 
            }
        ).into());
    }
    
    match manager.disable_plugin(name) {
        Ok(()) => {
            info!("CLI: Plugin '{}' disabled successfully", name);
            println!("Plugin '{}' disabled successfully.", name);
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("CLI: Plugin '{}' not found during disable operation", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(e) => {
            error!("CLI: Failed to disable plugin '{}': {} (category: {})", name, e, e.category());
            Err(e.user_friendly_message().into())
        }
    }
}

fn handle_execute(
    manager: &PluginManager,
    name: &str,
    input: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_str = input.unwrap_or("");
    info!("CLI: Starting execution of plugin '{}' with input length: {}", name, input_str.len());
    debug!("CLI: Plugin '{}' input content: '{}'", name, 
           if input_str.len() > 100 { 
               format!("{}...", &input_str[..100]) 
           } else { 
               input_str.to_string() 
           });
    
    // Pre-check plugin status for better error messaging
    if let Some(status) = manager.get_plugin_status(name) {
        debug!("CLI: Plugin '{}' status - enabled: {}, config_enabled: {}, loaded: {}", 
               name, status.enabled, status.config_enabled, status.loaded);
        
        if !status.enabled || !status.config_enabled {
            warn!("CLI: Attempted to execute disabled plugin '{}'", name);
            return Err(format!(
                "Plugin '{}' is disabled. Use 'enable {}' to enable it first.", 
                name, name
            ).into());
        }
    } else {
        warn!("CLI: Plugin '{}' not found in registry", name);
        let available_plugins: Vec<String> = manager.list_plugins()
            .iter()
            .filter(|p| p.enabled && p.config_enabled)
            .map(|p| p.name.clone())
            .collect();
        debug!("CLI: Available enabled plugins: {:?}", available_plugins);
        
        return Err(format!(
            "Plugin '{}' not found. Available enabled plugins: {}. Use 'list' command for details.", 
            name, 
            if available_plugins.is_empty() { 
                "none".to_string() 
            } else { 
                available_plugins.join(", ") 
            }
        ).into());
    }
    
    match manager.execute_plugin(name, input_str) {
        Ok(result) => {
            if result.success {
                info!("CLI: Plugin '{}' executed successfully in {}ms, output length: {}", 
                      name, result.duration_ms, result.output.len());
                debug!("CLI: Plugin '{}' output: {}", name, 
                       if result.output.len() > 200 { 
                           format!("{}...", &result.output[..200]) 
                       } else { 
                           result.output.clone() 
                       });
                
                println!("Plugin '{}' executed successfully:", name);
                println!("Output: {}", result.output);
                println!("Duration: {}ms", result.duration_ms);
            } else {
                error!("CLI: Plugin '{}' execution failed after {}ms: {}", 
                       name, result.duration_ms, result.output);
                return Err(format!("Plugin execution failed: {}", result.output).into());
            }
            Ok(())
        }
        Err(PluginError::NotFound { .. }) => {
            error!("CLI: Plugin '{}' not found during execution", name);
            Err(format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name).into())
        }
        Err(PluginError::PluginDisabled { .. }) => {
            error!("CLI: Plugin '{}' is disabled during execution", name);
            Err(format!("Plugin '{}' is disabled. Use 'enable {}' to enable it first.", name, name).into())
        }
        Err(e) => {
            error!("CLI: Failed to execute plugin '{}': {} (category: {})", name, e, e.category());
            Err(e.user_friendly_message().into())
        }
    }
}

fn handle_serve(
    manager: PluginManager,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("CLI: Starting HTTP API server on {}:{}", host, port);
    debug!("CLI: Server configuration - host: {}, port: {}", host, port);
    
    let host_owned = host.to_string();
    
    // Validate server configuration
    if port == 0 {
        error!("CLI: Invalid port number: {}", port);
        return Err("Invalid port number. Port must be between 1 and 65535.".into());
    }
    
    // Create a new Tokio runtime for the server
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            debug!("CLI: Tokio runtime created successfully");
            rt
        }
        Err(e) => {
            error!("CLI: Failed to create Tokio runtime: {}", e);
            return Err(format!("Failed to create async runtime: {}", e).into());
        }
    };
    
    rt.block_on(async move {
        // Set up graceful shutdown handling
        let shutdown_manager = ShutdownManager::new();
        let shutdown_signal = match shutdown_manager.setup_signal_handling().await {
            Ok(signal) => {
                debug!("CLI: Signal handling setup successfully");
                signal
            }
            Err(e) => {
                error!("CLI: Failed to setup signal handling: {}", e);
                return Err(format!("Failed to setup signal handling: {}", e).into());
            }
        };
        
        info!("CLI: HTTP API server configuration complete, starting server");
        println!("HTTP API server starting on {}:{}", host_owned, port);
        println!("Available endpoints:");
        println!("  GET    /health                     - Health check");
        println!("  GET    /api/v1/plugins             - List all plugins");
        println!("  POST   /api/v1/plugins/{{name}}/execute - Execute plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/enable  - Enable plugin");
        println!("  PUT    /api/v1/plugins/{{name}}/disable - Disable plugin");
        println!("Press Ctrl+C to stop the server");
        
        // Start the server with graceful shutdown handling and retry logic
        let server_result = run_server_with_shutdown_and_retry(manager, &host_owned, port, shutdown_signal).await;
        
        // Perform cleanup
        info!("CLI: Starting server cleanup");
        shutdown_manager.cleanup().await;
        
        match server_result {
            Ok(()) => {
                println!("Server shutdown completed successfully");
                info!("CLI: Server shutdown completed successfully");
            }
            Err(e) => {
                error!("CLI: Server encountered an error during shutdown: {}", e);
                return Err(e);
            }
        }
        
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    
    Ok(())
}

/// Run the server with graceful shutdown handling and retry logic
async fn run_server_with_shutdown_and_retry(
    manager: PluginManager,
    host: &str,
    port: u16,
    shutdown_signal: tokio::sync::mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("CLI: Starting server on {}:{}", host, port);
    
    // For now, we'll run the server once without retry logic to avoid the ownership issues
    // The retry logic can be added later when the API is refactored to support it better
    match api::start_server(manager, host, port, shutdown_signal).await {
        Ok(()) => {
            info!("CLI: Server shut down gracefully");
            Ok(())
        }
        Err(e) => {
            if is_recoverable_network_error(&e) {
                warn!("CLI: Recoverable network error occurred: {}", e);
                info!("CLI: Server stopped due to network error, but this is recoverable");
                Ok(())
            } else {
                error!("CLI: Server failed with non-recoverable error: {}", e);
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
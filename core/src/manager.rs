use crate::{
    Config, ConfigManager, PluginError, PluginRegistry, PluginResult,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};

/// Execution result with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub plugin: String,
    pub output: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// Plugin status information combining registry and configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub loaded: bool,
    pub path: std::path::PathBuf,
    pub config_enabled: bool,
}

/// Options for plugin execution with error recovery
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// Maximum number of retry attempts for transient failures
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Timeout for plugin execution (None for no timeout)
    pub timeout: Option<Duration>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            max_retries: 2,
            retry_delay: Duration::from_millis(100),
            timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl ExecutionOptions {
    /// Create execution options with no retries
    pub fn no_retry() -> Self {
        Self {
            max_retries: 1,
            retry_delay: Duration::from_millis(0),
            timeout: Some(Duration::from_secs(30)),
        }
    }
    
    /// Create execution options with aggressive retries
    pub fn aggressive_retry() -> Self {
        Self {
            max_retries: 5,
            retry_delay: Duration::from_millis(200),
            timeout: Some(Duration::from_secs(60)),
        }
    }
    
    /// Create execution options with no timeout
    pub fn no_timeout() -> Self {
        Self {
            max_retries: 2,
            retry_delay: Duration::from_millis(100),
            timeout: None,
        }
    }
}

/// High-level plugin manager that integrates registry and configuration
pub struct PluginManager {
    registry: PluginRegistry,
    config_manager: ConfigManager,
}

impl PluginManager {
    /// Create a new plugin manager with default configuration
    pub fn new() -> PluginResult<Self> {
        info!("Initializing plugin manager");
        
        let config_manager = ConfigManager::with_default_path()
            .map_err(|e| PluginError::config_error(format!("Failed to load configuration: {}", e)))?;
        
        let plugins_dir = config_manager.config().plugins_dir.clone();
        let registry = PluginRegistry::new(&plugins_dir);
        
        let mut manager = Self {
            registry,
            config_manager,
        };
        
        // Load plugins from the configured directory
        manager.load_plugins()?;
        
        info!("Plugin manager initialized successfully");
        Ok(manager)
    }

    /// Create a new plugin manager with custom configuration path
    pub fn with_config_path<P: AsRef<Path>>(config_path: P) -> PluginResult<Self> {
        info!("Initializing plugin manager with config: {:?}", config_path.as_ref());
        
        let config_manager = ConfigManager::new(config_path)
            .map_err(|e| PluginError::config_error(format!("Failed to load configuration: {}", e)))?;
        
        let plugins_dir = config_manager.config().plugins_dir.clone();
        let registry = PluginRegistry::new(&plugins_dir);
        
        let mut manager = Self {
            registry,
            config_manager,
        };
        
        // Load plugins from the configured directory
        manager.load_plugins()?;
        
        info!("Plugin manager initialized successfully");
        Ok(manager)
    }

    /// Load all plugins from the configured plugins directory
    pub fn load_plugins(&mut self) -> PluginResult<Vec<String>> {
        info!("Loading plugins from directory: {:?}", self.config_manager.plugins_dir());
        
        let loaded_plugins = self.registry.scan_and_load()?;
        
        // Sync plugin states with configuration
        for plugin_name in &loaded_plugins {
            let config_enabled = self.config_manager.is_plugin_enabled(plugin_name);
            if config_enabled {
                if let Err(e) = self.registry.enable_plugin(plugin_name) {
                    warn!("Failed to enable plugin '{}' from config: {}", plugin_name, e);
                }
            } else {
                if let Err(e) = self.registry.disable_plugin(plugin_name) {
                    warn!("Failed to disable plugin '{}' from config: {}", plugin_name, e);
                }
            }
        }
        
        info!("Successfully loaded {} plugins", loaded_plugins.len());
        Ok(loaded_plugins)
    }

    /// List all plugins with their status information
    pub fn list_plugins(&self) -> Vec<PluginStatus> {
        debug!("Listing all plugins");
        
        let plugin_infos = self.registry.list_plugins();
        let mut statuses = Vec::new();
        
        for info in plugin_infos {
            let config_enabled = self.config_manager.is_plugin_enabled(&info.name);
            let status = PluginStatus {
                name: info.name.clone(),
                version: info.version,
                description: info.description,
                enabled: info.enabled,
                loaded: info.loaded,
                path: info.path,
                config_enabled,
            };
            statuses.push(status);
        }
        
        debug!("Listed {} plugins", statuses.len());
        statuses
    }

    /// Get status information for a specific plugin
    pub fn get_plugin_status(&self, name: &str) -> Option<PluginStatus> {
        debug!("Getting status for plugin: {}", name);
        
        let info = self.registry.get_plugin_info(name)?;
        let config_enabled = self.config_manager.is_plugin_enabled(name);
        
        Some(PluginStatus {
            name: info.name.clone(),
            version: info.version,
            description: info.description,
            enabled: info.enabled,
            loaded: info.loaded,
            path: info.path,
            config_enabled,
        })
    }

    /// Enable a plugin and persist the state
    pub fn enable_plugin(&mut self, name: &str) -> PluginResult<()> {
        info!("Enabling plugin: {}", name);
        
        // Check if plugin exists
        if !self.registry.has_plugin(name) {
            error!("Cannot enable plugin '{}': not found", name);
            return Err(PluginError::NotFound {
                name: name.to_string(),
            });
        }
        
        // Enable in registry
        self.registry.enable_plugin(name)?;
        
        // Enable in configuration and persist
        self.config_manager.enable_plugin(name)
            .map_err(|e| PluginError::config_error(format!("Failed to persist plugin state: {}", e)))?;
        
        info!("Plugin '{}' enabled successfully", name);
        Ok(())
    }

    /// Disable a plugin and persist the state
    pub fn disable_plugin(&mut self, name: &str) -> PluginResult<()> {
        info!("Disabling plugin: {}", name);
        
        // Check if plugin exists
        if !self.registry.has_plugin(name) {
            error!("Cannot disable plugin '{}': not found", name);
            return Err(PluginError::NotFound {
                name: name.to_string(),
            });
        }
        
        // Disable in registry
        self.registry.disable_plugin(name)?;
        
        // Disable in configuration and persist
        self.config_manager.disable_plugin(name)
            .map_err(|e| PluginError::config_error(format!("Failed to persist plugin state: {}", e)))?;
        
        info!("Plugin '{}' disabled successfully", name);
        Ok(())
    }

    /// Execute a plugin with comprehensive error handling and result formatting
    pub fn execute_plugin(&self, name: &str, input: &str) -> PluginResult<ExecutionResult> {
        self.execute_plugin_with_options(name, input, ExecutionOptions::default())
    }
    
    /// Execute a plugin with configurable execution options
    pub fn execute_plugin_with_options(&self, name: &str, input: &str, options: ExecutionOptions) -> PluginResult<ExecutionResult> {
        info!("Executing plugin '{}' with input length: {} (timeout: {:?}, retries: {})", 
              name, input.len(), options.timeout, options.max_retries);
        
        let start_time = Instant::now();
        
        // Check if plugin exists first
        if !self.registry.has_plugin(name) {
            error!("Plugin '{}' not found for execution", name);
            return Err(PluginError::NotFound {
                name: name.to_string(),
            });
        }
        
        // Check if plugin is enabled
        if let Some(status) = self.get_plugin_status(name) {
            if !status.enabled || !status.config_enabled {
                warn!("Attempted to execute disabled plugin '{}'", name);
                return Err(PluginError::PluginDisabled {
                    name: name.to_string(),
                });
            }
        }
        
        // Execute the plugin with timeout and retry logic
        let result = if let Some(timeout) = options.timeout {
            self.execute_plugin_with_timeout(name, input, timeout, options.max_retries)
        } else {
            self.registry.execute_plugin_with_retry(name, input, options.max_retries, options.retry_delay)
        };
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(output) => {
                let execution_result = ExecutionResult {
                    plugin: name.to_string(),
                    output,
                    duration_ms: duration.as_millis() as u64,
                    success: true,
                };
                
                info!(
                    "Plugin '{}' executed successfully in {}ms, output length: {} (category: execution_success)",
                    name,
                    execution_result.duration_ms,
                    execution_result.output.len()
                );
                
                Ok(execution_result)
            }
            Err(e) => {
                let execution_result = ExecutionResult {
                    plugin: name.to_string(),
                    output: e.user_friendly_message(),
                    duration_ms: duration.as_millis() as u64,
                    success: false,
                };
                
                error!(
                    "Plugin '{}' execution failed after {}ms: {} (category: {})",
                    name, execution_result.duration_ms, e, e.category()
                );
                
                // Return the error result instead of propagating the error
                // This allows callers to get timing information even for failed executions
                Ok(execution_result)
            }
        }
    }
    
    /// Execute a plugin with timeout (simplified implementation)
    fn execute_plugin_with_timeout(&self, name: &str, input: &str, timeout: std::time::Duration, max_retries: u32) -> PluginResult<String> {
        // For now, we'll use a simple timeout approach without threading
        // This could be enhanced later with async execution or proper thread management
        let start_time = Instant::now();
        
        // Execute with retries, checking timeout between attempts
        for attempt in 1..=max_retries {
            if start_time.elapsed() >= timeout {
                warn!("Plugin '{}' execution timed out after {:?} (attempt {})", name, timeout, attempt);
                return Err(PluginError::timeout_error(format!("Plugin '{}' execution", name)));
            }
            
            match self.registry.execute_plugin(name, input) {
                Ok(result) => return Ok(result),
                Err(e) if attempt < max_retries && e.is_transient() => {
                    warn!("Transient error on attempt {}: {}. Retrying...", attempt, e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(PluginError::execution_failed("Maximum retries exceeded"))
    }

    /// Execute a plugin and return only the output (for backward compatibility)
    pub fn execute_plugin_simple(&self, name: &str, input: &str) -> PluginResult<String> {
        let result = self.execute_plugin(name, input)?;
        if result.success {
            Ok(result.output)
        } else {
            Err(PluginError::ExecutionFailed {
                message: result.output,
            })
        }
    }

    /// Check if a plugin exists and is loaded
    pub fn has_plugin(&self, name: &str) -> bool {
        self.registry.has_plugin(name)
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.registry.plugin_count()
    }

    /// Get configuration for a specific plugin
    pub fn get_plugin_config(&mut self, name: &str) -> &mut crate::PluginConfig {
        self.config_manager.get_plugin_config(name)
    }

    /// Set a plugin setting and persist it
    pub fn set_plugin_setting(
        &mut self,
        plugin_name: &str,
        key: &str,
        value: serde_json::Value,
    ) -> PluginResult<()> {
        info!("Setting plugin '{}' setting '{}' = {:?}", plugin_name, key, value);
        
        self.config_manager
            .set_plugin_setting(plugin_name, key, value)
            .map_err(|e| PluginError::config_error(format!("Failed to set plugin setting: {}", e)))?;
        
        Ok(())
    }

    /// Get a plugin setting
    pub fn get_plugin_setting(&self, plugin_name: &str, key: &str) -> Option<&serde_json::Value> {
        self.config_manager.get_plugin_setting(plugin_name, key)
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        self.config_manager.config()
    }

    /// Reload configuration and sync plugin states
    pub fn reload_config(&mut self) -> PluginResult<()> {
        info!("Reloading configuration");
        
        self.config_manager.reload()
            .map_err(|e| PluginError::config_error(format!("Failed to reload configuration: {}", e)))?;
        
        // Sync plugin states with the reloaded configuration
        let plugin_infos = self.registry.list_plugins();
        for info in plugin_infos {
            let config_enabled = self.config_manager.is_plugin_enabled(&info.name);
            if config_enabled != info.enabled {
                if config_enabled {
                    if let Err(e) = self.registry.enable_plugin(&info.name) {
                        warn!("Failed to enable plugin '{}' after config reload: {}", info.name, e);
                    }
                } else {
                    if let Err(e) = self.registry.disable_plugin(&info.name) {
                        warn!("Failed to disable plugin '{}' after config reload: {}", info.name, e);
                    }
                }
            }
        }
        
        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// Get plugins directory path
    pub fn plugins_dir(&self) -> &Path {
        self.config_manager.plugins_dir()
    }

    /// Get enabled plugins only
    pub fn list_enabled_plugins(&self) -> Vec<PluginStatus> {
        self.list_plugins()
            .into_iter()
            .filter(|status| status.enabled && status.config_enabled)
            .collect()
    }

    /// Get disabled plugins only
    pub fn list_disabled_plugins(&self) -> Vec<PluginStatus> {
        self.list_plugins()
            .into_iter()
            .filter(|status| !status.enabled || !status.config_enabled)
            .collect()
    }

    /// Batch enable multiple plugins
    pub fn enable_plugins(&mut self, plugin_names: &[String]) -> Vec<(String, PluginResult<()>)> {
        info!("Batch enabling {} plugins", plugin_names.len());
        
        let mut results = Vec::new();
        for name in plugin_names {
            let result = self.enable_plugin(name);
            results.push((name.clone(), result));
        }
        
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        info!("Batch enable completed: {}/{} plugins enabled successfully", success_count, plugin_names.len());
        
        results
    }

    /// Batch disable multiple plugins
    pub fn disable_plugins(&mut self, plugin_names: &[String]) -> Vec<(String, PluginResult<()>)> {
        info!("Batch disabling {} plugins", plugin_names.len());
        
        let mut results = Vec::new();
        for name in plugin_names {
            let result = self.disable_plugin(name);
            results.push((name.clone(), result));
        }
        
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        info!("Batch disable completed: {}/{} plugins disabled successfully", success_count, plugin_names.len());
        
        results
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default plugin manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (PluginManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        
        let manager = PluginManager::with_config_path(&config_path).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_manager_creation() {
        let (manager, _temp_dir) = create_test_manager();
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_list_plugins_empty() {
        let (manager, _temp_dir) = create_test_manager();
        let plugins = manager.list_plugins();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_plugin_not_found() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Test execute_plugin returns error for non-existent plugin
        let result = manager.execute_plugin("nonexistent", "test");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
        
        // Test execute_plugin_simple returns error
        let result = manager.execute_plugin_simple("nonexistent", "test");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }

    #[test]
    fn test_enable_disable_nonexistent_plugin() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let result = manager.enable_plugin("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
        
        let result = manager.disable_plugin("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }

    #[test]
    fn test_plugin_settings() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let value = serde_json::json!("test_value");
        manager.set_plugin_setting("test_plugin", "test_key", value.clone()).unwrap();
        
        let retrieved = manager.get_plugin_setting("test_plugin", "test_key");
        assert_eq!(retrieved, Some(&value));
    }

    #[test]
    fn test_batch_operations() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let plugin_names = vec!["plugin1".to_string(), "plugin2".to_string()];
        
        // Test batch enable (should fail since plugins don't exist)
        let results = manager.enable_plugins(&plugin_names);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, r)| r.is_err()));
        
        // Test batch disable (should fail since plugins don't exist)
        let results = manager.disable_plugins(&plugin_names);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, r)| r.is_err()));
    }

    #[test]
    fn test_execution_result_format() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Test execution of non-existent plugin now returns error
        let result = manager.execute_plugin("nonexistent", "test");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }

    #[test]
    fn test_config_access() {
        let (manager, _temp_dir) = create_test_manager();
        
        let config = manager.config();
        assert_eq!(config.log_level, "info");
        assert!(config.server.enabled);
    }

    #[test]
    fn test_plugins_dir() {
        let (manager, _temp_dir) = create_test_manager();
        
        let plugins_dir = manager.plugins_dir();
        assert!(plugins_dir.ends_with("target/plugins"));
    }
}
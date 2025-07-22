use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use log::{info, warn, error};

/// Main configuration structure for the plugin system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory where plugins are stored
    pub plugins_dir: PathBuf,
    /// Logging level for the system
    pub log_level: String,
    /// Server configuration for HTTP API
    pub server: ServerConfig,
    /// Per-plugin configuration settings
    pub plugins: HashMap<String, PluginConfig>,
}

/// Server configuration for HTTP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port number to listen on
    pub port: u16,
    /// Whether the server is enabled
    pub enabled: bool,
}

/// Configuration for individual plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin-specific settings as key-value pairs
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plugins_dir: PathBuf::from("target/plugins"),
            log_level: "info".to_string(),
            server: ServerConfig::default(),
            plugins: HashMap::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enabled: true,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
        }
    }
}

/// Configuration manager handles loading, saving, and validating configuration
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager with the specified config file path
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        let config = Self::load_or_create_default(&config_path)?;
        
        Ok(Self {
            config,
            config_path,
        })
    }

    /// Create a configuration manager with default config file location
    pub fn with_default_path() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Self::new(config_path)
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;
        Ok(current_dir.join("config.yaml"))
    }

    /// Load configuration from file or create default if it doesn't exist
    fn load_or_create_default(config_path: &Path) -> Result<Config> {
        if config_path.exists() {
            info!("Loading configuration from: {}", config_path.display());
            Self::load_from_file(config_path)
        } else {
            info!("Configuration file not found, creating default: {}", config_path.display());
            let config = Config::default();
            Self::save_to_file(&config, config_path)?;
            Ok(config)
        }
    }

    /// Load configuration from YAML file
    fn load_from_file(config_path: &Path) -> Result<Config> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))
            .unwrap_or_else(|e| {
                error!("Configuration parsing failed: {}. Using default configuration.", e);
                warn!("Invalid configuration will be backed up and replaced with defaults");
                
                // Backup the invalid config
                if let Err(backup_err) = Self::backup_invalid_config(config_path) {
                    error!("Failed to backup invalid config: {}", backup_err);
                }
                
                Config::default()
            });

        Self::validate_and_fix_config(config)
    }

    /// Backup invalid configuration file
    fn backup_invalid_config(config_path: &Path) -> Result<()> {
        let backup_path = config_path.with_extension("yaml.backup");
        fs::copy(config_path, &backup_path)
            .with_context(|| format!("Failed to backup config to: {}", backup_path.display()))?;
        info!("Invalid config backed up to: {}", backup_path.display());
        Ok(())
    }

    /// Validate configuration and apply defaults for invalid values
    fn validate_and_fix_config(mut config: Config) -> Result<Config> {
        // Validate and fix plugins directory
        if config.plugins_dir.as_os_str().is_empty() {
            warn!("Empty plugins directory, using default");
            config.plugins_dir = PathBuf::from("target/plugins");
        }

        // Validate and fix log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.log_level.as_str()) {
            warn!("Invalid log level '{}', using 'info'", config.log_level);
            config.log_level = "info".to_string();
        }

        // Validate and fix server configuration
        if config.server.host.is_empty() {
            warn!("Empty server host, using default");
            config.server.host = "127.0.0.1".to_string();
        }

        if config.server.port == 0 {
            warn!("Invalid server port {}, using default 8080", config.server.port);
            config.server.port = 8080;
        }

        Ok(config)
    }

    /// Save configuration to YAML file
    fn save_to_file(config: &Config, config_path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let yaml_content = serde_yaml::to_string(config)
            .context("Failed to serialize configuration to YAML")?;

        fs::write(config_path, yaml_content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        info!("Configuration saved to: {}", config_path.display());
        Ok(())
    }

    /// Get a reference to the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the current configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Save the current configuration to file
    pub fn save(&self) -> Result<()> {
        Self::save_to_file(&self.config, &self.config_path)
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_or_create_default(&self.config_path)?;
        Ok(())
    }

    /// Get plugin configuration, creating default if it doesn't exist
    pub fn get_plugin_config(&mut self, plugin_name: &str) -> &mut PluginConfig {
        self.config.plugins
            .entry(plugin_name.to_string())
            .or_insert_with(PluginConfig::default)
    }

    /// Enable a plugin and persist the change
    pub fn enable_plugin(&mut self, plugin_name: &str) -> Result<()> {
        let plugin_config = self.get_plugin_config(plugin_name);
        plugin_config.enabled = true;
        self.save()?;
        info!("Plugin '{}' enabled", plugin_name);
        Ok(())
    }

    /// Disable a plugin and persist the change
    pub fn disable_plugin(&mut self, plugin_name: &str) -> Result<()> {
        let plugin_config = self.get_plugin_config(plugin_name);
        plugin_config.enabled = false;
        self.save()?;
        info!("Plugin '{}' disabled", plugin_name);
        Ok(())
    }

    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, plugin_name: &str) -> bool {
        self.config.plugins
            .get(plugin_name)
            .map(|config| config.enabled)
            .unwrap_or(true) // Default to enabled if not configured
    }

    /// Set plugin setting and persist the change
    pub fn set_plugin_setting(&mut self, plugin_name: &str, key: &str, value: serde_json::Value) -> Result<()> {
        let plugin_config = self.get_plugin_config(plugin_name);
        plugin_config.settings.insert(key.to_string(), value);
        self.save()?;
        info!("Plugin '{}' setting '{}' updated", plugin_name, key);
        Ok(())
    }

    /// Get plugin setting
    pub fn get_plugin_setting(&self, plugin_name: &str, key: &str) -> Option<&serde_json::Value> {
        self.config.plugins
            .get(plugin_name)?
            .settings
            .get(key)
    }

    /// Update server configuration and persist the change
    pub fn update_server_config(&mut self, host: Option<String>, port: Option<u16>, enabled: Option<bool>) -> Result<()> {
        if let Some(host) = host {
            self.config.server.host = host;
        }
        if let Some(port) = port {
            self.config.server.port = port;
        }
        if let Some(enabled) = enabled {
            self.config.server.enabled = enabled;
        }
        self.save()?;
        info!("Server configuration updated");
        Ok(())
    }

    /// Get the plugins directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.config.plugins_dir
    }

    /// Update plugins directory and persist the change
    pub fn set_plugins_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.config.plugins_dir = path.as_ref().to_path_buf();
        self.save()?;
        info!("Plugins directory updated to: {}", self.config.plugins_dir.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.plugins_dir, PathBuf::from("target/plugins"));
        assert_eq!(config.log_level, "info");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enabled);
        assert!(config.plugins.is_empty());
    }

    #[test]
    fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let manager = ConfigManager::new(&config_path).unwrap();
        assert!(config_path.exists());
        assert_eq!(manager.config().log_level, "info");
    }

    #[test]
    fn test_plugin_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let mut manager = ConfigManager::new(&config_path).unwrap();
        
        // Plugin should be enabled by default
        assert!(manager.is_plugin_enabled("test_plugin"));
        
        // Disable plugin
        manager.disable_plugin("test_plugin").unwrap();
        assert!(!manager.is_plugin_enabled("test_plugin"));
        
        // Enable plugin
        manager.enable_plugin("test_plugin").unwrap();
        assert!(manager.is_plugin_enabled("test_plugin"));
    }

    #[test]
    fn test_plugin_settings() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let mut manager = ConfigManager::new(&config_path).unwrap();
        
        // Set plugin setting
        let value = serde_json::json!("test_value");
        manager.set_plugin_setting("test_plugin", "test_key", value.clone()).unwrap();
        
        // Get plugin setting
        let retrieved = manager.get_plugin_setting("test_plugin", "test_key");
        assert_eq!(retrieved, Some(&value));
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.log_level = "invalid".to_string();
        config.server.port = 0;
        config.server.host = "".to_string();
        
        let fixed_config = ConfigManager::validate_and_fix_config(config).unwrap();
        assert_eq!(fixed_config.log_level, "info");
        assert_eq!(fixed_config.server.port, 8080);
        assert_eq!(fixed_config.server.host, "127.0.0.1");
    }

    #[test]
    fn test_config_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        // Create manager and modify config
        {
            let mut manager = ConfigManager::new(&config_path).unwrap();
            manager.disable_plugin("test_plugin").unwrap();
            manager.set_plugin_setting("test_plugin", "key", serde_json::json!("value")).unwrap();
        }
        
        // Create new manager and verify persistence
        {
            let manager = ConfigManager::new(&config_path).unwrap();
            assert!(!manager.is_plugin_enabled("test_plugin"));
            assert_eq!(
                manager.get_plugin_setting("test_plugin", "key"),
                Some(&serde_json::json!("value"))
            );
        }
    }

    #[test]
    fn test_yaml_config_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        // Create a YAML config file
        let yaml_content = r#"
plugins_dir: "custom/plugins"
log_level: "debug"
server:
  host: "0.0.0.0"
  port: 9090
  enabled: true
plugins:
  example_plugin:
    enabled: false
    settings:
      timeout: 30
      retries: 3
"#;
        std::fs::write(&config_path, yaml_content).unwrap();
        
        // Load and verify configuration
        let manager = ConfigManager::new(&config_path).unwrap();
        let config = manager.config();
        
        assert_eq!(config.plugins_dir, PathBuf::from("custom/plugins"));
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert!(config.server.enabled);
        
        // Check plugin configuration
        assert!(!manager.is_plugin_enabled("example_plugin"));
        assert_eq!(
            manager.get_plugin_setting("example_plugin", "timeout"),
            Some(&serde_json::json!(30))
        );
        assert_eq!(
            manager.get_plugin_setting("example_plugin", "retries"),
            Some(&serde_json::json!(3))
        );
    }
}
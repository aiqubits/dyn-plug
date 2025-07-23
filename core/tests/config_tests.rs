use dyn_plug_core::{ConfigManager, Config, ServerConfig, PluginConfig};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_config_manager_creation_and_defaults() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let config_manager = ConfigManager::new(&config_path)?;
    
    // Test that config file was created
    assert!(config_path.exists());
    
    // Test default values
    let config = config_manager.config();
    assert_eq!(config.log_level, "info");
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert!(config.server.enabled);
    assert!(config.plugins.is_empty());
    
    Ok(())
}

#[test]
fn test_config_manager_plugin_operations() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test plugin enable/disable
    assert!(config_manager.is_plugin_enabled("test_plugin")); // Default is enabled
    
    config_manager.disable_plugin("test_plugin")?;
    assert!(!config_manager.is_plugin_enabled("test_plugin"));
    
    config_manager.enable_plugin("test_plugin")?;
    assert!(config_manager.is_plugin_enabled("test_plugin"));
    
    Ok(())
}

#[test]
fn test_config_manager_plugin_settings() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test setting and getting plugin settings
    let value = serde_json::json!("test_value");
    config_manager.set_plugin_setting("test_plugin", "test_key", value.clone())?;
    
    let retrieved = config_manager.get_plugin_setting("test_plugin", "test_key");
    assert_eq!(retrieved, Some(&value));
    
    // Test non-existent setting
    let non_existent = config_manager.get_plugin_setting("test_plugin", "non_existent");
    assert!(non_existent.is_none());
    
    // Test non-existent plugin
    let non_existent_plugin = config_manager.get_plugin_setting("non_existent", "key");
    assert!(non_existent_plugin.is_none());
    
    Ok(())
}

#[test]
fn test_config_manager_server_config() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test server configuration updates
    config_manager.update_server_config(
        Some("0.0.0.0".to_string()),
        Some(9090),
        Some(false)
    )?;
    
    let config = config_manager.config();
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert!(!config.server.enabled);
    
    Ok(())
}

#[test]
fn test_config_manager_plugins_dir() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test plugins directory access and update
    let original_dir = config_manager.plugins_dir();
    assert!(original_dir.ends_with("target/plugins"));
    
    let new_dir = temp_dir.path().join("custom_plugins");
    config_manager.set_plugins_dir(&new_dir)?;
    
    let updated_dir = config_manager.plugins_dir();
    assert_eq!(updated_dir, new_dir);
    
    Ok(())
}

#[test]
fn test_config_manager_persistence() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create and modify configuration
    {
        let mut config_manager = ConfigManager::new(&config_path)?;
        config_manager.disable_plugin("test_plugin")?;
        config_manager.set_plugin_setting("test_plugin", "key", serde_json::json!("value"))?;
        config_manager.update_server_config(None, Some(9090), None)?;
    }
    
    // Verify persistence by creating new manager
    {
        let config_manager = ConfigManager::new(&config_path)?;
        assert!(!config_manager.is_plugin_enabled("test_plugin"));
        assert_eq!(
            config_manager.get_plugin_setting("test_plugin", "key"),
            Some(&serde_json::json!("value"))
        );
        assert_eq!(config_manager.config().server.port, 9090);
    }
    
    Ok(())
}

#[test]
fn test_config_manager_reload() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Modify configuration externally
    let yaml_content = r#"
plugins_dir: "custom/plugins"
log_level: "debug"
server:
  host: "0.0.0.0"
  port: 9090
  enabled: true
plugins:
  external_plugin:
    enabled: false
    settings:
      timeout: 30
"#;
    fs::write(&config_path, yaml_content)?;
    
    // Reload and verify changes
    config_manager.reload()?;
    
    let config = config_manager.config();
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert!(!config_manager.is_plugin_enabled("external_plugin"));
    assert_eq!(
        config_manager.get_plugin_setting("external_plugin", "timeout"),
        Some(&serde_json::json!(30))
    );
    
    Ok(())
}

#[test]
fn test_config_validation_and_fixing() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create invalid configuration
    let invalid_yaml = r#"
plugins_dir: ""
log_level: "invalid_level"
server:
  host: ""
  port: 0
  enabled: true
plugins: {}
"#;
    fs::write(&config_path, invalid_yaml)?;
    
    // Configuration should be fixed automatically
    let config_manager = ConfigManager::new(&config_path)?;
    let config = config_manager.config();
    
    // Check that invalid values were fixed
    assert_eq!(config.plugins_dir.to_string_lossy(), "target/plugins");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    
    Ok(())
}

#[test]
fn test_config_manager_get_plugin_config() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test getting plugin config (should create default if not exists)
    let plugin_config = config_manager.get_plugin_config("new_plugin");
    assert!(plugin_config.enabled); // Default is enabled
    assert!(plugin_config.settings.is_empty());
    
    // Modify the config
    plugin_config.enabled = false;
    plugin_config.settings.insert("key".to_string(), serde_json::json!("value"));
    
    // Save and verify persistence
    config_manager.save()?;
    
    let new_manager = ConfigManager::new(&config_path)?;
    assert!(!new_manager.is_plugin_enabled("new_plugin"));
    assert_eq!(
        new_manager.get_plugin_setting("new_plugin", "key"),
        Some(&serde_json::json!("value"))
    );
    
    Ok(())
}

#[test]
fn test_config_default_path() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Test default path creation
    let default_path = ConfigManager::default_config_path()?;
    assert!(default_path.ends_with("config.yaml"));
    
    Ok(())
}

#[test]
fn test_config_structs_defaults() {
    let config = Config::default();
    assert_eq!(config.plugins_dir.to_string_lossy(), "target/plugins");
    assert_eq!(config.log_level, "info");
    assert!(config.plugins.is_empty());
    
    let server_config = ServerConfig::default();
    assert_eq!(server_config.host, "127.0.0.1");
    assert_eq!(server_config.port, 8080);
    assert!(server_config.enabled);
    
    let plugin_config = PluginConfig::default();
    assert!(plugin_config.enabled);
    assert!(plugin_config.settings.is_empty());
}
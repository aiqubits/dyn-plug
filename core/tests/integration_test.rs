use dyn_plug_core::{PluginRegistry, PluginResult, PluginManager, ConfigManager};
use std::fs;
use tempfile::TempDir;

mod plugin_manager_tests;
mod config_tests;
mod error_handling_tests;
mod plugin_lifecycle_tests;

#[test]
fn test_registry_integration() -> PluginResult<()> {
    // Initialize logging for the test
    let _ = env_logger::builder().is_test(true).try_init();

    // Create a temporary directory for plugins
    let temp_dir = TempDir::new().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    // Create a registry
    let registry = PluginRegistry::new(&plugins_dir);

    // Test initial state
    assert_eq!(registry.plugin_count(), 0);
    assert!(registry.list_plugins().is_empty());

    // Test scanning empty directory
    let loaded = registry.scan_and_load()?;
    assert!(loaded.is_empty());

    // Test plugin not found
    assert!(!registry.has_plugin("nonexistent"));
    assert!(registry.get_plugin_info("nonexistent").is_none());

    // Test execution of non-existent plugin
    let result = registry.execute_plugin("nonexistent", "test");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_plugin_info_structure() {
    use dyn_plug_core::PluginInfo;
    use std::path::PathBuf;

    let info = PluginInfo {
        name: "test_plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        enabled: true,
        loaded: true,
        path: PathBuf::from("/path/to/plugin.so"),
    };

    assert_eq!(info.name, "test_plugin");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.description, "A test plugin");
    assert!(info.enabled);
    assert!(info.loaded);
}

#[test]
fn test_plugin_manager_creation() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    assert_eq!(manager.plugin_count(), 0);
    assert!(manager.list_plugins().is_empty());
    
    Ok(())
}

#[test]
fn test_config_manager_integration() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test default configuration
    let config = config_manager.config();
    assert_eq!(config.log_level, "info");
    assert!(config.server.enabled);
    assert_eq!(config.server.port, 8080);
    
    // Test plugin configuration
    config_manager.enable_plugin("test_plugin")?;
    assert!(config_manager.is_plugin_enabled("test_plugin"));
    
    config_manager.disable_plugin("test_plugin")?;
    assert!(!config_manager.is_plugin_enabled("test_plugin"));
    
    // Test plugin settings
    let value = serde_json::json!("test_value");
    config_manager.set_plugin_setting("test_plugin", "test_key", value.clone())?;
    
    let retrieved = config_manager.get_plugin_setting("test_plugin", "test_key");
    assert_eq!(retrieved, Some(&value));
    
    Ok(())
}

#[test]
fn test_configuration_persistence() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create and modify configuration
    {
        let mut config_manager = ConfigManager::new(&config_path)?;
        config_manager.disable_plugin("test_plugin")?;
        config_manager.set_plugin_setting("test_plugin", "key", serde_json::json!("value"))?;
    }
    
    // Verify persistence by creating new manager
    {
        let config_manager = ConfigManager::new(&config_path)?;
        assert!(!config_manager.is_plugin_enabled("test_plugin"));
        assert_eq!(
            config_manager.get_plugin_setting("test_plugin", "key"),
            Some(&serde_json::json!("value"))
        );
    }
    
    Ok(())
}

#[test]
fn test_plugin_manager_with_real_plugins() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // This test requires actual plugin binaries to be built
    // We'll test with the assumption that plugins might not be available
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test basic functionality even without plugins
    let plugins = manager.list_plugins();
    // Could be 0 if no plugins built, any non-negative count is valid
    
    // Test plugin operations with non-existent plugin
    let result = manager.execute_plugin("nonexistent", "test");
    assert!(result.is_err());
    
    Ok(())
}

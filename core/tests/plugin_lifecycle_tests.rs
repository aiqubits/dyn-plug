use dyn_plug_core::{PluginManager, PluginRegistry, PluginResult, PluginError};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_plugin_lifecycle_without_real_plugins() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    
    // Create config with custom plugins directory
    let config_content = format!(r#"
plugins_dir: "{}"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {{}}
"#, plugins_dir.to_string_lossy());
    
    fs::write(&config_path, config_content).unwrap();
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test initial state
    assert_eq!(manager.plugin_count(), 0);
    assert!(manager.list_plugins().is_empty());
    
    // Test loading plugins from empty directory
    let loaded = manager.load_plugins()?;
    assert!(loaded.is_empty());
    
    Ok(())
}

#[test]
fn test_plugin_registry_lifecycle() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    
    let registry = PluginRegistry::new(&plugins_dir);
    
    // Test initial state
    assert_eq!(registry.plugin_count(), 0);
    assert!(registry.list_plugins().is_empty());
    
    // Test scanning empty directory
    let loaded = registry.scan_and_load()?;
    assert!(loaded.is_empty());
    
    // Test plugin operations on empty registry
    assert!(!registry.has_plugin("test_plugin"));
    assert!(registry.get_plugin_info("test_plugin").is_none());
    
    let result = registry.execute_plugin("test_plugin", "input");
    assert!(result.is_err());
    
    let result = registry.enable_plugin("test_plugin");
    assert!(result.is_err());
    
    let result = registry.disable_plugin("test_plugin");
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_plugin_state_management() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test plugin state tracking without actual plugins
    let plugins = manager.list_plugins();
    assert!(plugins.is_empty());
    
    let enabled_plugins = manager.list_enabled_plugins();
    assert!(enabled_plugins.is_empty());
    
    let disabled_plugins = manager.list_disabled_plugins();
    assert!(disabled_plugins.is_empty());
    
    Ok(())
}

#[test]
fn test_plugin_configuration_lifecycle() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test plugin configuration without actual plugins
    manager.set_plugin_setting("test_plugin", "timeout", serde_json::json!(30))?;
    manager.set_plugin_setting("test_plugin", "retries", serde_json::json!(3))?;
    
    // Verify settings are stored
    assert_eq!(
        manager.get_plugin_setting("test_plugin", "timeout"),
        Some(&serde_json::json!(30))
    );
    assert_eq!(
        manager.get_plugin_setting("test_plugin", "retries"),
        Some(&serde_json::json!(3))
    );
    
    // Test configuration persistence
    let config = manager.config();
    assert!(config.plugins.contains_key("test_plugin"));
    
    Ok(())
}

#[test]
fn test_plugin_directory_scanning() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    
    // Create some dummy files (not actual plugins)
    fs::write(plugins_dir.join("not_a_plugin.txt"), "dummy content").unwrap();
    fs::write(plugins_dir.join("README.md"), "# Plugins").unwrap();
    
    let registry = PluginRegistry::new(&plugins_dir);
    
    // Should not load non-plugin files
    let loaded = registry.scan_and_load()?;
    assert!(loaded.is_empty());
    
    Ok(())
}

#[test]
fn test_plugin_manager_with_custom_plugins_dir() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    let custom_plugins_dir = temp_dir.path().join("custom_plugins");
    fs::create_dir_all(&custom_plugins_dir).unwrap();
    
    // Create config with custom plugins directory
    let config_content = format!(r#"
plugins_dir: "{}"
log_level: "debug"
server:
  host: "0.0.0.0"
  port: 9090
  enabled: false
plugins: {{}}
"#, custom_plugins_dir.to_string_lossy());
    
    fs::write(&config_path, config_content).unwrap();
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Verify custom configuration is loaded
    let config = manager.config();
    assert_eq!(config.plugins_dir, custom_plugins_dir);
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert!(!config.server.enabled);
    
    // Verify plugins directory is correct
    assert_eq!(manager.plugins_dir(), custom_plugins_dir);
    
    Ok(())
}

#[test]
fn test_plugin_execution_result_structure() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test execution result for non-existent plugin
    let result = manager.execute_plugin("nonexistent", "test input");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    Ok(())
}

#[test]
fn test_plugin_manager_reload_functionality() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Modify configuration externally
    let new_config = r#"
plugins_dir: "target/plugins"
log_level: "debug"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins:
  test_plugin:
    enabled: false
    settings:
      timeout: 60
"#;
    fs::write(&config_path, new_config).unwrap();
    
    // Reload configuration
    manager.reload_config()?;
    
    // Verify changes are applied
    let config = manager.config();
    assert_eq!(config.log_level, "debug");
    assert!(!manager.get_plugin_config("test_plugin").enabled);
    assert_eq!(
        manager.get_plugin_setting("test_plugin", "timeout"),
        Some(&serde_json::json!(60))
    );
    
    Ok(())
}

#[test]
fn test_plugin_batch_operations_lifecycle() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    let plugin_names = vec![
        "plugin1".to_string(),
        "plugin2".to_string(),
        "plugin3".to_string(),
    ];
    
    // Test batch enable (should fail for non-existent plugins)
    let results = manager.enable_plugins(&plugin_names);
    assert_eq!(results.len(), 3);
    for (_name, result) in &results {
        assert!(result.is_err());
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }
    
    // Test batch disable (should fail for non-existent plugins)
    let results = manager.disable_plugins(&plugin_names);
    assert_eq!(results.len(), 3);
    for (_name, result) in &results {
        assert!(result.is_err());
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }
    
    Ok(())
}

#[test]
fn test_plugin_registry_with_retry_logic() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    
    let registry = PluginRegistry::new(&plugins_dir);
    
    // Test retry execution (should still fail for non-existent plugin)
    let result = registry.execute_plugin_with_retry(
        "nonexistent",
        "test",
        3,
        std::time::Duration::from_millis(10)
    );
    assert!(result.is_err());
    
    Ok(())
}
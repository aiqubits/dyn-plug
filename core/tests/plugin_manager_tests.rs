use dyn_plug_core::{PluginManager, PluginResult, PluginError, ExecutionOptions};
use tempfile::TempDir;
use std::time::Duration;

#[test]
fn test_plugin_manager_basic_operations() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test initial state
    assert_eq!(manager.plugin_count(), 0);
    assert!(manager.list_plugins().is_empty());
    assert!(manager.list_enabled_plugins().is_empty());
    assert!(manager.list_disabled_plugins().is_empty());
    
    // Test plugin not found operations
    assert!(!manager.has_plugin("nonexistent"));
    assert!(manager.get_plugin_status("nonexistent").is_none());
    
    let result = manager.enable_plugin("nonexistent");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    let result = manager.disable_plugin("nonexistent");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    let result = manager.execute_plugin("nonexistent", "test");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    Ok(())
}

#[test]
fn test_plugin_manager_configuration_integration() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test configuration access
    let config = manager.config();
    assert_eq!(config.log_level, "info");
    assert!(config.server.enabled);
    
    // Test plugin settings
    let value = serde_json::json!("test_value");
    manager.set_plugin_setting("test_plugin", "test_key", value.clone())?;
    
    let retrieved = manager.get_plugin_setting("test_plugin", "test_key");
    assert_eq!(retrieved, Some(&value));
    
    // Test plugins directory
    let plugins_dir = manager.plugins_dir();
    assert!(plugins_dir.ends_with("target/plugins"));
    
    Ok(())
}

#[test]
fn test_plugin_manager_execution_options() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test execution with different options
    let options = ExecutionOptions::no_retry();
    let result = manager.execute_plugin_with_options("nonexistent", "test", options);
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    let options = ExecutionOptions::aggressive_retry();
    let result = manager.execute_plugin_with_options("nonexistent", "test", options);
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    let options = ExecutionOptions::no_timeout();
    let result = manager.execute_plugin_with_options("nonexistent", "test", options);
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    Ok(())
}

#[test]
fn test_plugin_manager_batch_operations() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    let plugin_names = vec!["plugin1".to_string(), "plugin2".to_string()];
    
    // Test batch enable (should fail since plugins don't exist)
    let results = manager.enable_plugins(&plugin_names);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, r)| r.is_err()));
    
    // Test batch disable (should fail since plugins don't exist)
    let results = manager.disable_plugins(&plugin_names);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, r)| r.is_err()));
    
    Ok(())
}

#[test]
fn test_plugin_manager_config_reload() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test config reload
    manager.reload_config()?;
    
    // Configuration should still be accessible after reload
    let config = manager.config();
    assert_eq!(config.log_level, "info");
    
    Ok(())
}

#[test]
fn test_execution_options_creation() {
    let default_options = ExecutionOptions::default();
    assert_eq!(default_options.max_retries, 2);
    assert_eq!(default_options.retry_delay, Duration::from_millis(100));
    assert_eq!(default_options.timeout, Some(Duration::from_secs(30)));
    
    let no_retry_options = ExecutionOptions::no_retry();
    assert_eq!(no_retry_options.max_retries, 1);
    
    let aggressive_options = ExecutionOptions::aggressive_retry();
    assert_eq!(aggressive_options.max_retries, 5);
    assert_eq!(aggressive_options.retry_delay, Duration::from_millis(200));
    
    let no_timeout_options = ExecutionOptions::no_timeout();
    assert!(no_timeout_options.timeout.is_none());
}

#[test]
fn test_plugin_manager_simple_execution() -> PluginResult<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test simple execution method
    let result = manager.execute_plugin_simple("nonexistent", "test");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    Ok(())
}
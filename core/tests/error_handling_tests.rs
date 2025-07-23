use dyn_plug_core::{PluginError, PluginManager, PluginRegistry, ConfigManager};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_plugin_error_types() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Test NotFound error
    let not_found_error = PluginError::NotFound {
        name: "test_plugin".to_string(),
    };
    assert_eq!(not_found_error.category(), "not_found");
    assert!(not_found_error.user_friendly_message().contains("test_plugin"));
    assert!(!not_found_error.is_transient());
    
    // Test ExecutionFailed error
    let execution_error = PluginError::ExecutionFailed {
        message: "Test execution failure".to_string(),
    };
    assert_eq!(execution_error.category(), "execution_failed");
    assert!(execution_error.user_friendly_message().contains("execution failure"));
    assert!(!execution_error.is_transient());
    
    // Test ConfigError
    let config_error = PluginError::ConfigError {
        message: "Test config error".to_string(),
    };
    assert_eq!(config_error.category(), "config_error");
    assert!(config_error.user_friendly_message().contains("configuration"));
    assert!(!config_error.is_transient());
    
    // Test PluginDisabled error
    let disabled_error = PluginError::PluginDisabled {
        name: "test_plugin".to_string(),
    };
    assert_eq!(disabled_error.category(), "plugin_disabled");
    assert!(disabled_error.user_friendly_message().contains("disabled"));
    assert!(!disabled_error.is_transient());
}

#[test]
fn test_plugin_error_creation_helpers() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Test helper methods
    let config_error = PluginError::config_error("Test message");
    assert!(matches!(config_error, PluginError::ConfigError { .. }));
    
    let execution_error = PluginError::execution_failed("Test message");
    assert!(matches!(execution_error, PluginError::ExecutionFailed { .. }));
    
    let timeout_error = PluginError::timeout_error("Test timeout");
    assert!(matches!(timeout_error, PluginError::TimeoutError { .. }));
    
    let resource_error = PluginError::resource_exhausted("Test resource");
    assert!(matches!(resource_error, PluginError::ResourceExhausted { .. }));
}

#[test]
fn test_plugin_manager_error_handling() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test enable non-existent plugin
    let result = manager.enable_plugin("nonexistent");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    // Test disable non-existent plugin
    let result = manager.disable_plugin("nonexistent");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    // Test execute non-existent plugin
    let result = manager.execute_plugin("nonexistent", "test");
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
    
    Ok(())
}

#[test]
fn test_registry_error_handling() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir)?;
    
    let registry = PluginRegistry::new(&plugins_dir);
    
    // Test execute non-existent plugin
    let result = registry.execute_plugin("nonexistent", "test");
    assert!(result.is_err());
    
    // Test enable non-existent plugin
    let result = registry.enable_plugin("nonexistent");
    assert!(result.is_err());
    
    // Test disable non-existent plugin
    let result = registry.disable_plugin("nonexistent");
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_config_manager_error_handling() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Test with invalid directory path
    let invalid_path = "/invalid/path/that/does/not/exist/config.yaml";
    let _result = ConfigManager::new(invalid_path);
    // This might succeed if the directory can be created, or fail - both are valid
    // The important thing is that it doesn't panic
    
    // Test with read-only directory (if we can create one)
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create a valid config manager first
    let mut config_manager = ConfigManager::new(&config_path)?;
    
    // Test operations that should succeed
    config_manager.enable_plugin("test_plugin")?;
    config_manager.disable_plugin("test_plugin")?;
    config_manager.set_plugin_setting("test_plugin", "key", serde_json::json!("value"))?;
    
    Ok(())
}

#[test]
fn test_error_display_and_debug() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let error = PluginError::NotFound {
        name: "test_plugin".to_string(),
    };
    
    // Test Display implementation
    let display_str = format!("{}", error);
    assert!(display_str.contains("test_plugin"));
    
    // Test Debug implementation
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NotFound"));
    assert!(debug_str.contains("test_plugin"));
}

#[test]
fn test_transient_error_detection() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Test non-transient errors
    let not_found = PluginError::NotFound { name: "test".to_string() };
    assert!(!not_found.is_transient());
    
    let execution_failed = PluginError::ExecutionFailed { message: "test".to_string() };
    assert!(!execution_failed.is_transient());
    
    let config_error = PluginError::ConfigError { message: "test".to_string() };
    assert!(!config_error.is_transient());
    
    let disabled = PluginError::PluginDisabled { name: "test".to_string() };
    assert!(!disabled.is_transient());
    
    // Test potentially transient errors
    let timeout = PluginError::TimeoutError { operation: "test".to_string() };
    assert!(timeout.is_transient());
    
    let resource_exhausted = PluginError::ResourceExhausted { resource: "test".to_string() };
    assert!(resource_exhausted.is_transient());
}

#[test]
fn test_error_categories() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let errors = vec![
        (PluginError::NotFound { name: "test".to_string() }, "not_found"),
        (PluginError::ExecutionFailed { message: "test".to_string() }, "execution_failed"),
        (PluginError::ConfigError { message: "test".to_string() }, "config_error"),
        (PluginError::PluginDisabled { name: "test".to_string() }, "plugin_disabled"),
        (PluginError::TimeoutError { operation: "test".to_string() }, "timeout_error"),
        (PluginError::ResourceExhausted { resource: "test".to_string() }, "resource_exhausted"),
    ];
    
    for (error, expected_category) in errors {
        assert_eq!(error.category(), expected_category);
    }
}

#[test]
fn test_user_friendly_messages() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let not_found = PluginError::NotFound { name: "test_plugin".to_string() };
    let message = not_found.user_friendly_message();
    assert!(message.contains("test_plugin"));
    assert!(message.contains("not found"));
    
    let execution_failed = PluginError::ExecutionFailed { message: "Custom error".to_string() };
    let message = execution_failed.user_friendly_message();
    assert!(message.contains("execution failed") || message.contains("Custom error"));
    
    let config_error = PluginError::ConfigError { message: "Config issue".to_string() };
    let message = config_error.user_friendly_message();
    assert!(message.contains("configuration"));
    
    let disabled = PluginError::PluginDisabled { name: "test_plugin".to_string() };
    let message = disabled.user_friendly_message();
    assert!(message.contains("test_plugin"));
    assert!(message.contains("disabled"));
}

#[test]
fn test_error_chain_compatibility() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let error = PluginError::ExecutionFailed { message: "Test error".to_string() };
    
    // Test that error can be used with anyhow
    let _anyhow_error: anyhow::Error = error.into();
    
    // Test that error implements std::error::Error
    let error = PluginError::NotFound { name: "test".to_string() };
    let _std_error: &dyn std::error::Error = &error;
}
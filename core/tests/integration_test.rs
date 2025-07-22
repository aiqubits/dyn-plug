use dyn_plug_core::{PluginRegistry, PluginResult};
use std::fs;
use tempfile::TempDir;

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

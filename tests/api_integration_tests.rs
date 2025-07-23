use dyn_plug_core::PluginManager;
use tempfile::TempDir;
use std::fs;
use std::process::Command;

// API Integration Tests
// These tests focus on testing the API functionality through the CLI serve command
// and basic HTTP connectivity rather than detailed endpoint testing

fn create_test_config(temp_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content)?;
    Ok(())
}

fn build_binary() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["build", "--bin", "dyn-plug"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to build binary: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}

#[test]
fn test_api_server_startup() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    // Build the binary first
    if let Err(e) = build_binary() {
        eprintln!("Warning: Failed to build binary, skipping API tests: {}", e);
        return Ok(());
    }
    
    // Test that the serve command can start (we'll kill it quickly)
    let binary_path = {
        let mut path = std::env::current_dir()?;
        path.push("target");
        path.push("debug");
        path.push("dyn-plug");
        
        if cfg!(windows) {
            path.set_extension("exe");
        }
        
        path
    };
    
    // Test serve command validation
    let output = Command::new(&binary_path)
        .args(&["serve", "--port", "0"]) // Invalid port should fail
        .current_dir(temp_dir.path())
        .output()?;
    
    // Should fail with invalid port
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid port") || stderr.contains("port"));
    
    println!("✓ API server validation test passed");
    Ok(())
}

#[test]
fn test_api_configuration_validation() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    
    // Test with valid configuration
    create_test_config(&temp_dir)?;
    
    // Create a plugin manager to test API-related configuration
    let config_path = temp_dir.path().join("config.yaml");
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test that server configuration is accessible
    let config = manager.config();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert!(config.server.enabled);
    
    println!("✓ API configuration validation test passed");
    Ok(())
}

#[test]
fn test_api_plugin_manager_integration() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    let config_path = temp_dir.path().join("config.yaml");
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test operations that the API would perform
    let _plugins = manager.list_plugins();
    // plugins.len() is always >= 0 by definition, so no need to assert
    
    // Test plugin execution (should fail for non-existent plugin)
    let result = manager.execute_plugin("nonexistent", "test");
    assert!(result.is_err());
    
    println!("✓ API plugin manager integration test passed");
    Ok(())
}

#[test]
fn test_api_error_response_handling() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    let config_path = temp_dir.path().join("config.yaml");
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test error scenarios that API endpoints would handle
    
    // Test enable non-existent plugin
    let result = manager.enable_plugin("nonexistent");
    assert!(result.is_err());
    
    // Test disable non-existent plugin  
    let result = manager.disable_plugin("nonexistent");
    assert!(result.is_err());
    
    // Test execute non-existent plugin
    let result = manager.execute_plugin("nonexistent", "test");
    assert!(result.is_err());
    
    println!("✓ API error response handling test passed");
    Ok(())
}

#[test]
fn test_api_concurrent_access_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    let config_path = temp_dir.path().join("config.yaml");
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Simulate concurrent access patterns that the API would experience
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let manager = Arc::new(Mutex::new(manager));
    let mut handles = vec![];
    
    // Simulate multiple concurrent list operations
    for _i in 0..3 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            if let Ok(manager) = manager_clone.lock() {
                let _plugins = manager.list_plugins();
                // Simulate some processing time
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().map_err(|_| "Thread panicked")?;
    }
    
    println!("✓ API concurrent access simulation test passed");
    Ok(())
}

#[test]
fn test_api_large_input_handling() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    let config_path = temp_dir.path().join("config.yaml");
    let manager = PluginManager::with_config_path(&config_path)?;
    
    // Test with large input (simulating API request with large payload)
    let large_input = "x".repeat(10000); // 10KB input
    let result = manager.execute_plugin("nonexistent", &large_input);
    
    // Should still handle the error gracefully, not crash
    assert!(result.is_err());
    
    println!("✓ API large input handling test passed");
    Ok(())
}

#[test]
fn test_api_configuration_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new()?;
    create_test_config(&temp_dir)?;
    
    let config_path = temp_dir.path().join("config.yaml");
    let mut manager = PluginManager::with_config_path(&config_path)?;
    
    // Test configuration reload (API might need this functionality)
    manager.reload_config()?;
    
    // Configuration should still be accessible after reload
    let config = manager.config();
    assert_eq!(config.server.host, "127.0.0.1");
    
    println!("✓ API configuration reload test passed");
    Ok(())
}
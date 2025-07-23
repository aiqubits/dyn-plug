use std::process::Command;
use std::fs;
use tempfile::TempDir;
use std::path::PathBuf;

/// Helper function to build the CLI binary for testing
fn build_cli_binary() -> PathBuf {
    let output = Command::new("cargo")
        .args(&["build", "--bin", "dyn-plug"])
        .output()
        .expect("Failed to build CLI binary");
    
    if !output.status.success() {
        panic!("Failed to build CLI binary: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    let mut binary_path = std::env::current_dir().unwrap();
    binary_path.push("target");
    binary_path.push("debug");
    binary_path.push("dyn-plug");
    
    if cfg!(windows) {
        binary_path.set_extension("exe");
    }
    
    binary_path
}

/// Helper function to run CLI command with temporary config
fn run_cli_command(args: &[&str], config_dir: Option<&std::path::Path>) -> std::process::Output {
    let binary_path = build_cli_binary();
    let mut cmd = Command::new(&binary_path);
    cmd.args(args);
    
    if let Some(dir) = config_dir {
        cmd.current_dir(dir);
    }
    
    cmd.output().expect("Failed to execute CLI command")
}

#[test]
fn test_cli_list_command() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["list"], Some(temp_dir.path()));
    
    // Should succeed even with no plugins
    assert!(output.status.success(), "CLI list command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain "No plugins found" or plugin listing headers
    assert!(stdout.contains("No plugins found") || stdout.contains("Available plugins"));
}

#[test]
fn test_cli_enable_nonexistent_plugin() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["enable", "nonexistent_plugin"], Some(temp_dir.path()));
    
    // Should fail with appropriate error message
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Plugin 'nonexistent_plugin' not found"));
}

#[test]
fn test_cli_disable_nonexistent_plugin() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["disable", "nonexistent_plugin"], Some(temp_dir.path()));
    
    // Should fail with appropriate error message
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Plugin 'nonexistent_plugin' not found"));
}

#[test]
fn test_cli_execute_nonexistent_plugin() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["execute", "nonexistent_plugin", "--input", "test"], Some(temp_dir.path()));
    
    // Should fail with appropriate error message
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Plugin 'nonexistent_plugin' not found"));
}

#[test]
fn test_cli_execute_without_input() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["execute", "nonexistent_plugin"], Some(temp_dir.path()));
    
    // Should fail with plugin not found (input defaults to empty string)
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Plugin 'nonexistent_plugin' not found"));
}

#[test]
fn test_cli_help_command() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let output = run_cli_command(&["--help"], None);
    
    // Should succeed and show help
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A pluggable service system"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("enable"));
    assert!(stdout.contains("disable"));
    assert!(stdout.contains("execute"));
    assert!(stdout.contains("serve"));
}

#[test]
fn test_cli_version_command() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let output = run_cli_command(&["--version"], None);
    
    // Should succeed and show version
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_cli_with_custom_config() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let custom_plugins_dir = temp_dir.path().join("custom_plugins");
    fs::create_dir_all(&custom_plugins_dir).unwrap();
    
    // Create a config with custom plugins directory
    let config_content = format!(r#"
plugins_dir: "{}"
log_level: "debug"
server:
  host: "0.0.0.0"
  port: 9090
  enabled: false
plugins: {{}}
"#, custom_plugins_dir.to_string_lossy());
    
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    let output = run_cli_command(&["list"], Some(temp_dir.path()));
    
    // Should succeed with custom config
    assert!(output.status.success(), "CLI failed with custom config: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_cli_invalid_command() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let output = run_cli_command(&["invalid_command"], None);
    
    // Should fail with error about invalid command
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Clap should provide error about unrecognized subcommand
    assert!(stderr.contains("unrecognized subcommand") || stderr.contains("invalid") || stderr.contains("error"));
}

#[test]
fn test_cli_serve_command_validation() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create a basic config file
    let config_content = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: 8080
  enabled: true
plugins: {}
"#;
    fs::write(temp_dir.path().join("config.yaml"), config_content).unwrap();
    
    // Test serve command with invalid port
    let output = run_cli_command(&["serve", "--port", "0"], Some(temp_dir.path()));
    
    // Should fail with invalid port error
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid port") || stderr.contains("port"));
}

#[test]
fn test_cli_configuration_persistence() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // First run should create default config
    let output = run_cli_command(&["list"], Some(temp_dir.path()));
    assert!(output.status.success());
    
    // Config file should be created
    assert!(config_path.exists());
    
    // Read the created config
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("plugins_dir"));
    assert!(config_content.contains("log_level"));
    assert!(config_content.contains("server"));
}

#[test]
fn test_cli_error_handling_with_corrupted_config() {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create an invalid YAML config
    let invalid_config = r#"
plugins_dir: "target/plugins"
log_level: "info"
server:
  host: "127.0.0.1"
  port: invalid_port
  enabled: true
plugins: {
"#; // Intentionally malformed YAML
    
    fs::write(temp_dir.path().join("config.yaml"), invalid_config).unwrap();
    
    let output = run_cli_command(&["list"], Some(temp_dir.path()));
    
    // Should handle corrupted config gracefully (might succeed with defaults or fail gracefully)
    // The important thing is it shouldn't panic
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should either succeed with defaults or provide meaningful error
    if !output.status.success() {
        assert!(stderr.contains("config") || stderr.contains("yaml") || stderr.contains("parse"));
    }
}
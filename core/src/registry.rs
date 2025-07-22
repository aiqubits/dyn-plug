use crate::{Plugin, PluginError, PluginResult};
use libloading::{Library, Symbol};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub loaded: bool,
    pub path: PathBuf,
}

/// A loaded plugin with its associated library
struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
    #[allow(dead_code)] // Keep library alive to prevent unloading
    library: Library,
    info: PluginInfo,
}

/// Plugin registry that manages dynamic loading and storage of plugins
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    plugins_dir: PathBuf,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new<P: AsRef<Path>>(plugins_dir: P) -> Self {
        let plugins_dir = plugins_dir.as_ref().to_path_buf();
        info!("Initializing plugin registry with directory: {:?}", plugins_dir);
        
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugins_dir,
        }
    }

    /// Scan the plugins directory and load all available plugins with retry logic
    pub fn scan_and_load(&self) -> PluginResult<Vec<String>> {
        self.scan_and_load_with_retry(3, std::time::Duration::from_millis(500))
    }
    
    /// Scan the plugins directory and load all available plugins with configurable retry
    pub fn scan_and_load_with_retry(&self, max_retries: u32, retry_delay: std::time::Duration) -> PluginResult<Vec<String>> {
        info!("Scanning plugins directory: {:?}", self.plugins_dir);
        
        if !self.plugins_dir.exists() {
            warn!("Plugins directory does not exist: {:?}", self.plugins_dir);
            std::fs::create_dir_all(&self.plugins_dir)?;
            info!("Created plugins directory: {:?}", self.plugins_dir);
            return Ok(Vec::new());
        }

        let mut loaded_plugins = Vec::new();
        let mut failed_plugins = Vec::new();
        
        let entries = std::fs::read_dir(&self.plugins_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if self.is_plugin_library(&path) {
                debug!("Found potential plugin library: {:?}", path);
                
                // Try loading with retry logic for transient failures
                let mut last_error = None;
                let mut loaded = false;
                
                for attempt in 1..=max_retries {
                    match self.load_plugin_from_path(&path) {
                        Ok(plugin_name) => {
                            if attempt > 1 {
                                info!("Successfully loaded plugin '{}' on attempt {}", plugin_name, attempt);
                            }
                            loaded_plugins.push(plugin_name);
                            loaded = true;
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < max_retries && last_error.as_ref().unwrap().is_transient() {
                                warn!("Transient error loading plugin from {:?} (attempt {}): {}. Retrying in {:?}...", 
                                      path, attempt, last_error.as_ref().unwrap(), retry_delay);
                                std::thread::sleep(retry_delay);
                            } else {
                                break;
                            }
                        }
                    }
                }
                
                if !loaded {
                    if let Some(error) = last_error {
                        error!("Failed to load plugin from {:?} after {} attempts: {}", path, max_retries, error);
                        failed_plugins.push((path.clone(), error));
                    }
                }
            }
        }

        if !failed_plugins.is_empty() {
            warn!("Failed to load {} plugins, but continuing with {} successful loads", 
                  failed_plugins.len(), loaded_plugins.len());
            
            // Log detailed failure information
            for (path, error) in &failed_plugins {
                error!("Plugin load failure: {:?} - {} (category: {})", 
                       path, error, error.category());
            }
        }

        info!("Successfully loaded {} plugins ({} failed)", loaded_plugins.len(), failed_plugins.len());
        Ok(loaded_plugins)
    }

    /// Load a specific plugin from a file path
    pub fn load_plugin_from_path<P: AsRef<Path>>(&self, path: P) -> PluginResult<String> {
        let path = path.as_ref();
        info!("Loading plugin from: {:?}", path);

        // Load the dynamic library
        let library = unsafe {
            Library::new(path).map_err(|e| {
                error!("Failed to load library {:?}: {}", path, e);
                PluginError::LoadingFailed { source: e }
            })?
        };

        // Get the plugin registration function
        let register_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = unsafe {
            library.get(b"register_plugin").map_err(|e| {
                error!("Failed to find register_plugin symbol in {:?}: {}", path, e);
                PluginError::RegistrationFailed {
                    message: format!("Missing register_plugin symbol in {:?}", path),
                }
            })?
        };

        // Call the registration function to get the plugin instance
        let plugin_ptr = unsafe { register_fn() };
        if plugin_ptr.is_null() {
            error!("Plugin registration returned null pointer for {:?}", path);
            return Err(PluginError::RegistrationFailed {
                message: format!("Plugin registration returned null for {:?}", path),
            });
        }

        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Extract plugin metadata
        let name = plugin.name().to_string();
        let version = plugin.version().to_string();
        let description = plugin.description().to_string();

        debug!("Loaded plugin: {} v{} - {}", name, version, description);

        let plugin_info = PluginInfo {
            name: name.clone(),
            version,
            description,
            enabled: true, // Default to enabled
            loaded: true,
            path: path.to_path_buf(),
        };

        let loaded_plugin = LoadedPlugin {
            plugin,
            library,
            info: plugin_info,
        };

        // Store the plugin in the registry
        {
            let mut plugins = self.plugins.write().unwrap();
            if plugins.contains_key(&name) {
                warn!("Plugin {} already exists, replacing with new version", name);
            }
            plugins.insert(name.clone(), loaded_plugin);
        }

        info!("Successfully registered plugin: {}", name);
        Ok(name)
    }

    /// Get plugin information by name
    pub fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).map(|p| p.info.clone())
    }

    /// Get information for all plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.info.clone()).collect()
    }

    /// Execute a plugin by name with retry logic for transient failures
    pub fn execute_plugin(&self, name: &str, input: &str) -> PluginResult<String> {
        self.execute_plugin_with_retry(name, input, 2, std::time::Duration::from_millis(100))
    }
    
    /// Execute a plugin by name with configurable retry logic
    pub fn execute_plugin_with_retry(&self, name: &str, input: &str, max_retries: u32, retry_delay: std::time::Duration) -> PluginResult<String> {
        debug!("Executing plugin: {} with input length: {} (max_retries: {})", name, input.len(), max_retries);
        
        let plugins = self.plugins.read().unwrap();
        let loaded_plugin = plugins.get(name).ok_or_else(|| {
            error!("Plugin not found: {}", name);
            PluginError::NotFound {
                name: name.to_string(),
            }
        })?;

        if !loaded_plugin.info.enabled {
            warn!("Attempted to execute disabled plugin: {}", name);
            return Err(PluginError::PluginDisabled {
                name: name.to_string(),
            });
        }

        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            match loaded_plugin.plugin.execute(input) {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Plugin {} executed successfully on attempt {}, output length: {}", 
                              name, attempt, result.len());
                    } else {
                        debug!("Plugin {} executed successfully, output length: {}", name, result.len());
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let plugin_error = PluginError::execution_failed(&e);
                    last_error = Some(plugin_error);
                    
                    if attempt < max_retries && self.is_execution_error_transient(&e) {
                        warn!("Transient execution error for plugin {} (attempt {}): {}. Retrying in {:?}...", 
                              name, attempt, e, retry_delay);
                        std::thread::sleep(retry_delay);
                    } else {
                        error!("Plugin {} execution failed on attempt {}: {}", name, attempt, e);
                        break;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
    
    /// Check if a plugin execution error is transient and worth retrying
    fn is_execution_error_transient(&self, error: &Box<dyn std::error::Error>) -> bool {
        let error_str = error.to_string().to_lowercase();
        
        // Common transient execution errors
        error_str.contains("timeout") ||
        error_str.contains("temporary") ||
        error_str.contains("busy") ||
        error_str.contains("resource temporarily unavailable") ||
        error_str.contains("would block") ||
        error_str.contains("interrupted")
    }

    /// Enable a plugin
    pub fn enable_plugin(&self, name: &str) -> PluginResult<()> {
        info!("Enabling plugin: {}", name);
        
        let mut plugins = self.plugins.write().unwrap();
        let loaded_plugin = plugins.get_mut(name).ok_or_else(|| {
            error!("Cannot enable plugin, not found: {}", name);
            PluginError::NotFound {
                name: name.to_string(),
            }
        })?;

        loaded_plugin.info.enabled = true;
        info!("Plugin {} enabled successfully", name);
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&self, name: &str) -> PluginResult<()> {
        info!("Disabling plugin: {}", name);
        
        let mut plugins = self.plugins.write().unwrap();
        let loaded_plugin = plugins.get_mut(name).ok_or_else(|| {
            error!("Cannot disable plugin, not found: {}", name);
            PluginError::NotFound {
                name: name.to_string(),
            }
        })?;

        loaded_plugin.info.enabled = false;
        info!("Plugin {} disabled successfully", name);
        Ok(())
    }

    /// Check if a plugin exists in the registry
    pub fn has_plugin(&self, name: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(name)
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        let plugins = self.plugins.read().unwrap();
        plugins.len()
    }

    /// Check if a file is a potential plugin library based on its extension
    fn is_plugin_library(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let extension = path.extension().and_then(OsStr::to_str);
        match extension {
            Some("so") => true,    // Linux
            Some("dll") => true,   // Windows
            Some("dylib") => true, // macOS
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        assert_eq!(registry.plugin_count(), 0);
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        let result = registry.scan_and_load().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");
        let registry = PluginRegistry::new(&nonexistent_path);
        let result = registry.scan_and_load().unwrap();
        assert!(result.is_empty());
        assert!(nonexistent_path.exists()); // Should be created
    }

    #[test]
    fn test_is_plugin_library() {
        let temp_dir = TempDir::new().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());

        // Create test files
        let so_file = temp_dir.path().join("test.so");
        let dll_file = temp_dir.path().join("test.dll");
        let dylib_file = temp_dir.path().join("test.dylib");
        let txt_file = temp_dir.path().join("test.txt");

        fs::write(&so_file, "").unwrap();
        fs::write(&dll_file, "").unwrap();
        fs::write(&dylib_file, "").unwrap();
        fs::write(&txt_file, "").unwrap();

        assert!(registry.is_plugin_library(&so_file));
        assert!(registry.is_plugin_library(&dll_file));
        assert!(registry.is_plugin_library(&dylib_file));
        assert!(!registry.is_plugin_library(&txt_file));
    }

    #[test]
    fn test_plugin_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        
        let result = registry.execute_plugin("nonexistent", "test");
        assert!(matches!(result, Err(PluginError::NotFound { .. })));
    }
}
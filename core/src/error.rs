use thiserror::Error;

/// Core error types for the plugin system
/// 
/// This enum defines all the possible errors that can occur during
/// plugin operations, providing structured error handling throughout
/// the system.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin was not found in the registry
    #[error("Plugin not found: {name}")]
    NotFound { name: String },
    
    /// Failed to load a plugin from a dynamic library
    #[error("Plugin loading failed: {source}")]
    LoadingFailed {
        #[from]
        source: libloading::Error,
    },
    
    /// Plugin execution failed
    #[error("Plugin execution failed: {message}")]
    ExecutionFailed { message: String },
    
    /// Configuration-related error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    /// Plugin is disabled and cannot be executed
    #[error("Plugin is disabled: {name}")]
    PluginDisabled { name: String },
    
    /// Plugin registration failed
    #[error("Plugin registration failed: {message}")]
    RegistrationFailed { message: String },
    
    /// I/O error occurred during plugin operations
    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    
    /// Serialization/deserialization error
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
}

impl PluginError {
    /// Create a new ExecutionFailed error
    pub fn execution_failed<E>(error: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self::ExecutionFailed {
            message: error.to_string(),
        }
    }
    
    /// Create a new ConfigError
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }
    
    /// Create a new RegistrationFailed error
    pub fn registration_failed<S: Into<String>>(message: S) -> Self {
        Self::RegistrationFailed {
            message: message.into(),
        }
    }
}

/// Result type alias for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
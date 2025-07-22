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
    
    /// Network-related error (for HTTP API operations)
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    /// Timeout error during plugin operations
    #[error("Operation timed out: {operation}")]
    TimeoutError { operation: String },
    
    /// Resource exhaustion error
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
    
    /// Temporary failure that may be retried
    #[error("Temporary failure: {message}")]
    TemporaryFailure { message: String },
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
    
    /// Create a new NetworkError
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
    
    /// Create a new TimeoutError
    pub fn timeout_error<S: Into<String>>(operation: S) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
        }
    }
    
    /// Create a new ResourceExhausted error
    pub fn resource_exhausted<S: Into<String>>(resource: S) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
        }
    }
    
    /// Create a new TemporaryFailure error
    pub fn temporary_failure<S: Into<String>>(message: S) -> Self {
        Self::TemporaryFailure {
            message: message.into(),
        }
    }
    
    /// Check if this error is transient and worth retrying
    pub fn is_transient(&self) -> bool {
        match self {
            PluginError::IoError { source } => {
                matches!(source.kind(), 
                    std::io::ErrorKind::PermissionDenied |
                    std::io::ErrorKind::TimedOut |
                    std::io::ErrorKind::Interrupted |
                    std::io::ErrorKind::WouldBlock |
                    std::io::ErrorKind::ConnectionRefused |
                    std::io::ErrorKind::ConnectionAborted |
                    std::io::ErrorKind::NotConnected
                )
            }
            PluginError::ConfigError { message } => {
                message.contains("temporary") || 
                message.contains("lock") ||
                message.contains("busy") ||
                message.contains("in use")
            }
            PluginError::NetworkError { .. } => true,
            PluginError::TimeoutError { .. } => true,
            PluginError::ResourceExhausted { .. } => true,
            PluginError::TemporaryFailure { .. } => true,
            _ => false,
        }
    }
    
    /// Get a user-friendly error message with recovery suggestions
    pub fn user_friendly_message(&self) -> String {
        match self {
            PluginError::NotFound { name } => {
                format!("Plugin '{}' not found. Use 'list' command to see available plugins.", name)
            }
            PluginError::LoadingFailed { source } => {
                format!("Failed to load plugin: {}. Check plugin file integrity and permissions.", source)
            }
            PluginError::ExecutionFailed { message } => {
                format!("Plugin execution failed: {}. Check plugin input format and try again.", message)
            }
            PluginError::ConfigError { message } => {
                format!("Configuration error: {}. Check configuration file permissions and format.", message)
            }
            PluginError::PluginDisabled { name } => {
                format!("Plugin '{}' is disabled. Use 'enable {}' to enable it first.", name, name)
            }
            PluginError::RegistrationFailed { message } => {
                format!("Plugin registration failed: {}. Plugin may be corrupted or incompatible.", message)
            }
            PluginError::IoError { source } => {
                format!("I/O error: {}. Check file permissions and disk space.", source)
            }
            PluginError::SerializationError { source } => {
                format!("Data format error: {}. Check input data format.", source)
            }
            PluginError::NetworkError { message } => {
                format!("Network error: {}. Check network connectivity and try again.", message)
            }
            PluginError::TimeoutError { operation } => {
                format!("Operation timed out: {}. Try again or increase timeout settings.", operation)
            }
            PluginError::ResourceExhausted { resource } => {
                format!("Resource exhausted: {}. Free up resources and try again.", resource)
            }
            PluginError::TemporaryFailure { message } => {
                format!("Temporary failure: {}. Please try again in a moment.", message)
            }
        }
    }
    
    /// Get the error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            PluginError::NotFound { .. } => "not_found",
            PluginError::LoadingFailed { .. } => "loading_failed",
            PluginError::ExecutionFailed { .. } => "execution_failed",
            PluginError::ConfigError { .. } => "config_error",
            PluginError::PluginDisabled { .. } => "plugin_disabled",
            PluginError::RegistrationFailed { .. } => "registration_failed",
            PluginError::IoError { .. } => "io_error",
            PluginError::SerializationError { .. } => "serialization_error",
            PluginError::NetworkError { .. } => "network_error",
            PluginError::TimeoutError { .. } => "timeout_error",
            PluginError::ResourceExhausted { .. } => "resource_exhausted",
            PluginError::TemporaryFailure { .. } => "temporary_failure",
        }
    }
}

/// Result type alias for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
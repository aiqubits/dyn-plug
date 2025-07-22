pub mod plugin;
pub mod error;
pub mod registry;
pub mod config;
pub mod manager;

pub use plugin::Plugin;
pub use error::{PluginError, PluginResult};
pub use registry::{PluginRegistry, PluginInfo};
pub use config::{Config, ConfigManager, PluginConfig, ServerConfig};
pub use manager::{PluginManager, PluginStatus, ExecutionResult, ExecutionOptions};

// Re-export commonly used types
pub use anyhow::Result;
use std::error::Error;

/// The core trait that all plugins must implement
///
/// This trait defines the standard interface for all plugins in the system.
/// Plugins must be thread-safe (Send + Sync) to support concurrent execution.
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin
    fn name(&self) -> &str;

    /// Returns the version of the plugin
    fn version(&self) -> &str;

    /// Returns a description of what the plugin does
    fn description(&self) -> &str;

    /// Executes the plugin with the given input and returns the result
    ///
    /// # Arguments
    /// * `input` - The input string to process
    ///
    /// # Returns
    /// * `Ok(String)` - The processed output
    /// * `Err(Box<dyn Error>)` - An error if execution fails
    fn execute(&self, input: &str) -> Result<String, Box<dyn Error>>;
}

/// Macro to simplify plugin registration
///
/// This macro generates the required `register_plugin` function that the
/// plugin system uses to load plugins from dynamic libraries.
///
/// # Example
/// ```rust
/// use dyn_plug_core::{Plugin, register_plugin};
///
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn name(&self) -> &str { "my_plugin" }
///     fn version(&self) -> &str { "1.0.0" }
///     fn description(&self) -> &str { "A sample plugin" }
///     fn execute(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
///         Ok(format!("Processed: {}", input))
///     }
/// }
///
/// impl MyPlugin {
///     pub fn new() -> Self {
///         Self
///     }
/// }
///
/// register_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn register_plugin() -> *mut dyn $crate::Plugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }
    };
}

# Technology Stack

## Core Technologies

- **Language**: Rust (Edition 2021)
- **Build System**: Cargo workspace with multiple crates
- **Plugin System**: Dynamic library loading via `libloading`
- **Web Framework**: Actix Web for HTTP API
- **Async Runtime**: Tokio
- **CLI Framework**: Clap with derive macros
- **Logging**: `log` + `env_logger`
- **Serialization**: Serde (JSON)

## Key Dependencies

- `anyhow` - Error handling
- `thiserror` - Custom error types
- `libloading` - Dynamic library loading
- `actix-web` - HTTP server
- `tokio` - Async runtime
- `clap` - CLI argument parsing
- `serde` + `serde_json` - Serialization
- `log` + `env_logger` - Logging
- `ctrlc` - Signal handling
- `fs_extra` - File operations
- `rayon` - Parallel computing

## Build Commands

```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Build specific plugin
cargo build --package plugin_a --release

# # Build and deploy all plugins
# ./build_plugins.sh

# Run the main application
cargo run

# Run with specific command
cargo run -- list
cargo run -- serve --port 8080
```

## Plugin Development

- Plugins are separate crates in `plugins/` directory
- Must implement the `Plugin` trait from core crate
- Use `register_plugin!` macro for plugin registration
- Built as dynamic libraries (cdylib crate type)
- Loaded at runtime via `
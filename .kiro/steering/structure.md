# Project Structure

## Workspace Layout

```
dyn-plug/
├── Cargo.toml              # Workspace root manifest
├── Cargo.lock              # Dependency lock file
├── build_plugins.sh        # Plugin build/deploy script
├── src/                    # Main application
│   ├── main.rs            # CLI entry point
│   └── api.rs             # HTTP API implementation
├── core/                   # Core plugin system library
│   ├── Cargo.toml         # Core crate manifest
│   └── src/
│       ├── lib.rs         # Public API exports
│       ├── plugin.rs      # Plugin trait definition
│       ├── registry.rs    # Plugin registry/loader
│       ├── manager.rs     # High-level plugin manager
│       ├── config.rs      # Configuration management
│       └── service.rs     # Plugin service orchestration
└── plugins/               # Plugin implementations
    ├── plugin_a/          # Example string processing plugin
    ├── plugin_b/          # Example numeric processing plugin
    └── plugin_c/          # Example JSON processing plugin
```

## Architecture Patterns

### Core Library (`core/`)
- **Plugin Trait**: Defines the interface all plugins must implement
- **Registry**: Low-level plugin loading and management
- **Manager**: High-level plugin lifecycle and configuration
- **Service**: Event-driven plugin orchestration
- **Config**: JSON-based configuration persistence

### Plugin Structure
Each plugin follows this pattern:
```
plugins/plugin_name/
├── Cargo.toml          # Plugin manifest (crate-type = ["cdylib"])
└── src/
    └── lib.rs          # Plugin implementation + register_plugin! macro
```

### Main Application (`src/`)
- **CLI Interface**: Comprehensive command-line tool
- **HTTP API**: REST endpoints for remote management
- **Service Mode**: Daemon mode with event handling

## Configuration Files

- `config.yaml` - System and plugin configuration
- `target/plugins/` - Compiled plugin binaries directory
- Plugin configs stored in main config under plugin names

## Naming Conventions

- Plugin crates: `plugin_*` (lowercase with underscores)
- Plugin names: Match crate name without prefix
- Dynamic libraries: `plugin_*.{so|dll|dylib}`
- Configuration keys: snake_case
- CLI commands: kebab-case where applicable
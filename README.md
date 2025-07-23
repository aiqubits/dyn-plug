# DynPlug

**DynPlug** is a pluggable service system written in Rust that provides a dynamic plugin architecture for loading and executing external modules at runtime.

## Key Features

- **Dynamic Plugin Loading**: Load plugins from shared libraries (.so/.dll/.dylib) at runtime
- **Plugin Management**: Enable/disable, configure, and manage plugin lifecycle
- **CLI Interface**: Comprehensive command-line interface for plugin operations
- **HTTP API**: REST API for remote plugin management and execution
- **Service Mode**: Run as a daemon with HTTP API endpoint
- **Configuration Management**: YAML-based configuration with per-plugin settings

## Use Cases

- Modular service architecture
- Runtime extensibility
- Plugin-based automation systems
- Microservice orchestration
- Dynamic feature loading

## [Testing doc](./TESTING.md)

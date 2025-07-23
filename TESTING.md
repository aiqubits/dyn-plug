# DynPlug Testing Guide

This document describes the comprehensive testing strategy for the DynPlug plugin system.

## Test Structure

The testing suite is organized into multiple layers to ensure comprehensive coverage:

### 1. Core Library Tests (`core/tests/`)

#### Integration Tests (`core/tests/integration_test.rs`)
- Basic plugin registry functionality
- Plugin manager creation and initialization
- Configuration manager integration
- Configuration persistence testing

#### Plugin Manager Tests (`core/tests/plugin_manager_tests.rs`)
- Plugin manager basic operations
- Configuration integration
- Execution options testing
- Batch operations
- Configuration reload functionality

#### Configuration Tests (`core/tests/config_tests.rs`)
- Configuration manager creation and defaults
- Plugin enable/disable operations
- Plugin settings management
- Server configuration updates
- Configuration persistence and reload
- YAML configuration loading
- Configuration validation and error recovery

#### Error Handling Tests (`core/tests/error_handling_tests.rs`)
- Plugin error types and categorization
- Error creation helpers
- Plugin manager error scenarios
- Registry error handling
- Configuration manager error handling
- Error display and debugging
- Transient error detection
- User-friendly error messages

#### Plugin Lifecycle Tests (`core/tests/plugin_lifecycle_tests.rs`)
- Plugin lifecycle without real plugins
- Plugin registry lifecycle
- Plugin state management
- Plugin configuration lifecycle
- Plugin directory scanning
- Custom plugins directory configuration
- Plugin execution result structure
- Configuration reload functionality
- Batch operations lifecycle
- Registry retry logic

### 2. CLI Integration Tests (`tests/cli_integration_tests.rs`)

Tests the command-line interface functionality:

- **List Command**: Tests plugin listing with various configurations
- **Enable/Disable Commands**: Tests plugin state management via CLI
- **Execute Command**: Tests plugin execution through CLI
- **Help and Version**: Tests CLI help and version information
- **Configuration Handling**: Tests CLI with custom configurations
- **Error Scenarios**: Tests CLI error handling and user feedback
- **Configuration Persistence**: Tests that CLI operations persist configuration changes

### 3. HTTP API Integration Tests (`tests/api_integration_tests.rs`)

Tests the REST API endpoints:

- **Health Endpoints**: Tests `/health` and `/api/v1/health` endpoints
- **Plugin Listing**: Tests `GET /api/v1/plugins` endpoint
- **Plugin Execution**: Tests `POST /api/v1/plugins/{name}/execute` endpoint
- **Plugin Management**: Tests `PUT /api/v1/plugins/{name}/enable` and `PUT /api/v1/plugins/{name}/disable` endpoints
- **Error Handling**: Tests API error responses and status codes
- **Request Validation**: Tests API input validation
- **Response Format**: Tests consistent API response structure
- **Concurrent Requests**: Tests API behavior under concurrent load
- **Custom Configuration**: Tests API with custom configurations

### 4. End-to-End Tests (`tests/end_to_end_tests.rs`)

Comprehensive tests that exercise the entire system:

- **Complete Plugin Lifecycle**: Tests plugin loading, execution, and management
- **Configuration Persistence**: Tests configuration changes across application restarts
- **Error Scenarios**: Tests system behavior in various error conditions
- **Real Plugin Execution**: Tests with actual compiled plugins (when available)
- **Plugin Enable/Disable Cycle**: Tests complete plugin state management
- **Concurrent Operations**: Tests system behavior under concurrent access
- **Configuration Validation**: Tests configuration validation and recovery

## Test Categories

### Unit Tests
- Individual component functionality
- Error handling and edge cases
- Configuration validation
- Data structure behavior

### Integration Tests
- Component interaction
- Plugin manager and registry integration
- Configuration system integration
- API endpoint functionality

### End-to-End Tests
- Complete user workflows
- CLI command execution
- API request/response cycles
- Plugin lifecycle management
- Configuration persistence

## Running Tests

### Quick Test Run
```bash
# Run all tests
cargo test

# Run specific test category
cargo test --test integration_test
cargo test --test cli_integration_tests
cargo test --test api_integration_tests
cargo test --test end_to_end_tests
```

### Comprehensive Test Run
```bash
# Run the comprehensive test suite
./run_integration_tests.sh
```

This script will:
1. Check prerequisites
2. Build the application and plugins
3. Run all test categories in order
4. Provide detailed output and error reporting
5. Clean up test artifacts

### Individual Test Categories
```bash
# Core library tests
cargo test --package dyn-plug-core

# Plugin manager tests
cargo test plugin_manager_tests --package dyn-plug-core

# Configuration tests
cargo test config_tests --package dyn-plug-core

# Error handling tests
cargo test error_handling_tests --package dyn-plug-core

# CLI integration tests
cargo test --test cli_integration_tests

# API integration tests
cargo test --test api_integration_tests

# End-to-end tests
cargo test --test end_to_end_tests
```

## Test Environment Setup

### Prerequisites
- Rust toolchain (cargo)
- Plugin build dependencies
- Temporary directory access for test isolation

### Plugin Building
Tests that require real plugins will attempt to build them automatically. If plugin building fails, these tests will skip plugin-specific functionality but still test the system's error handling.

### Configuration Isolation
All tests use temporary directories and configurations to ensure test isolation and prevent interference between test runs.

## Test Coverage Areas

### ✅ Implemented Test Coverage

1. **Plugin Registry**
   - Plugin loading and scanning
   - Plugin information retrieval
   - Plugin execution
   - Plugin enable/disable operations
   - Error handling for missing plugins

2. **Plugin Manager**
   - Manager initialization
   - Plugin lifecycle management
   - Configuration integration
   - Batch operations
   - Execution options and retry logic

3. **Configuration System**
   - Configuration loading and saving
   - Plugin settings management
   - Server configuration
   - Configuration validation and recovery
   - Persistence across restarts

4. **Error Handling**
   - Comprehensive error types
   - Error categorization and user-friendly messages
   - Transient error detection
   - Error propagation and handling

5. **CLI Interface**
   - All CLI commands (list, enable, disable, execute, serve)
   - Command-line argument parsing
   - Error reporting and user feedback
   - Configuration file handling

6. **HTTP API**
   - All API endpoints
   - Request/response handling
   - Error status codes
   - JSON serialization/deserialization
   - Concurrent request handling

7. **End-to-End Workflows**
   - Complete plugin lifecycle
   - Configuration persistence
   - Real plugin execution (when available)
   - Error recovery scenarios

### Test Quality Features

- **Isolation**: Each test uses temporary directories and configurations
- **Error Handling**: Tests verify both success and failure scenarios
- **Real-world Scenarios**: Tests simulate actual usage patterns
- **Performance**: Tests include timeout handling and concurrent operations
- **Robustness**: Tests handle missing plugins and invalid configurations gracefully

## Continuous Integration

The test suite is designed to work in CI environments:

- Tests handle missing plugins gracefully
- Temporary directories are used for isolation
- Comprehensive error reporting
- Exit codes indicate test success/failure
- Detailed logging for debugging

## Debugging Tests

### Verbose Output
```bash
cargo test -- --nocapture
```

### Specific Test
```bash
cargo test test_name -- --nocapture
```

### Test Logging
Tests initialize logging with:
```rust
let _ = env_logger::builder().is_test(true).try_init();
```

Set `RUST_LOG=debug` for detailed logging during tests.

## Contributing to Tests

When adding new functionality:

1. Add unit tests for individual components
2. Add integration tests for component interactions
3. Add end-to-end tests for complete workflows
4. Update this documentation
5. Ensure tests handle error scenarios
6. Use temporary directories for test isolation
7. Make tests robust to missing dependencies
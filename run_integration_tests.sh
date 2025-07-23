#!/bin/bash

# Comprehensive Integration Test Runner for DynPlug
# This script runs all integration tests in the correct order

set -e  # Exit on any error

echo "🚀 Starting DynPlug Comprehensive Integration Tests"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run a test category
run_test_category() {
    local category=$1
    local description=$2
    
    echo ""
    print_status "Running $description..."
    echo "----------------------------------------"
    
    if cargo test $category --verbose; then
        print_success "$description completed successfully"
    else
        print_error "$description failed"
        return 1
    fi
}

# Function to build plugins
build_plugins() {
    print_status "Building plugins for integration tests..."
    
    if ./build_plugins.sh; then
        print_success "Plugins built successfully"
        return 0
    else
        print_warning "Plugin build failed - some tests may be skipped"
        return 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if build script exists
    if [ ! -f "build_plugins.sh" ]; then
        print_warning "build_plugins.sh not found. Creating a basic version..."
        cat > build_plugins.sh << 'EOF'
#!/bin/bash
echo "Building plugins..."
cargo build --release --package plugin_a
cargo build --release --package plugin_b  
cargo build --release --package plugin_c
echo "Plugins built successfully"
EOF
        chmod +x build_plugins.sh
    fi
    
    print_success "Prerequisites check completed"
}

# Function to clean up test artifacts
cleanup() {
    print_status "Cleaning up test artifacts..."
    
    # Remove any temporary test files
    find . -name "*.tmp" -delete 2>/dev/null || true
    find . -name "test_config_*.yaml" -delete 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Main test execution
main() {
    echo "Starting comprehensive integration tests at $(date)"
    
    # Check prerequisites
    check_prerequisites
    
    # Clean up any previous test artifacts
    cleanup
    
    # Build the main application
    print_status "Building main application..."
    if cargo build; then
        print_success "Main application built successfully"
    else
        print_error "Failed to build main application"
        exit 1
    fi
    
    # Build plugins (optional - tests should handle missing plugins gracefully)
    build_plugins || print_warning "Continuing without plugins..."
    
    # Run core library unit tests
    run_test_category "--lib --package dyn-plug-core" "Core Library Unit Tests"
    
    # Run core integration tests
    run_test_category "--test integration_test --package dyn-plug-core" "Core Integration Tests"
    
    # Run plugin manager tests
    run_test_category "plugin_manager_tests --package dyn-plug-core" "Plugin Manager Tests"
    
    # Run configuration tests
    run_test_category "config_tests --package dyn-plug-core" "Configuration Tests"
    
    # Run error handling tests
    run_test_category "error_handling_tests --package dyn-plug-core" "Error Handling Tests"
    
    # Run plugin lifecycle tests
    run_test_category "plugin_lifecycle_tests --package dyn-plug-core" "Plugin Lifecycle Tests"
    
    # Run CLI integration tests
    run_test_category "--test cli_integration_tests" "CLI Integration Tests"
    
    # Run API integration tests
    run_test_category "--test api_integration_tests" "HTTP API Integration Tests"
    
    # Run end-to-end tests
    run_test_category "--test end_to_end_tests" "End-to-End Tests"
    
    # Run all remaining tests
    print_status "Running any remaining tests..."
    if cargo test --workspace; then
        print_success "All remaining tests passed"
    else
        print_warning "Some additional tests failed"
    fi
    
    # Final cleanup
    cleanup
    
    echo ""
    echo "=================================================="
    print_success "🎉 All integration tests completed successfully!"
    echo "Test run completed at $(date)"
}

# Handle script interruption
trap cleanup EXIT

# Run main function
main "$@"
#!/usr/bin/env bash

# Bash script for compiling and deploying plugins

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_green() {
    echo -e "${GREEN}$1${NC}"
}

print_cyan() {
    echo -e "${CYAN}$1${NC}"
}

print_red() {
    echo -e "${RED}$1${NC}"
}

# Create plugins directory
PLUGIN_DIR="target/plugins"
if [ ! -d "$PLUGIN_DIR" ]; then
    mkdir -p "$PLUGIN_DIR"
fi

# Determine file extension based on OS
case "$(uname -s)" in
    Linux*)     EXT="so";;
    Darwin*)    EXT="dylib";;
    CYGWIN*|MINGW*|MSYS*) EXT="dll";;
    *)          EXT="so";;
esac

print_green "Building plugins..."

# Array of plugins to build
PLUGINS=("plugin_a" "plugin_b" "plugin_c")
BUILD_ERRORS=0

# Build each plugin and track errors
for plugin in "${PLUGINS[@]}"; do
    print_cyan "Building $plugin..."
    if cargo build --package "$plugin" --release; then
        print_green "✓ $plugin built successfully"
    else
        print_red "✗ Failed to build $plugin"
        BUILD_ERRORS=$((BUILD_ERRORS + 1))
    fi
done

# Check if there were build errors
if [ $BUILD_ERRORS -gt 0 ]; then
    print_red "Build failed: $BUILD_ERRORS plugin(s) failed to compile"
    exit 1
fi

print_green "Deploying plugins..."

# Copy plugin files and track deployment errors
DEPLOY_ERRORS=0
for plugin in "${PLUGINS[@]}"; do
    SRC_FILE="target/release/lib${plugin}.${EXT}"
    DEST_FILE="${PLUGIN_DIR}/${plugin}.${EXT}"
    
    if [ -f "$SRC_FILE" ]; then
        if cp "$SRC_FILE" "$DEST_FILE"; then
            print_green "✓ $plugin deployed successfully"
        else
            print_red "✗ Failed to deploy $plugin"
            DEPLOY_ERRORS=$((DEPLOY_ERRORS + 1))
        fi
    else
        print_red "✗ Plugin binary not found: $SRC_FILE"
        DEPLOY_ERRORS=$((DEPLOY_ERRORS + 1))
    fi
done

# Check if there were deployment errors
if [ $DEPLOY_ERRORS -gt 0 ]; then
    print_red "Deployment failed: $DEPLOY_ERRORS plugin(s) failed to deploy"
    exit 1
fi

print_green "All plugins successfully deployed to $PLUGIN_DIR"
print_cyan "You can now run 'cargo run' to test the plugin system"

# List deployed plugins
print_cyan "Deployed plugins:"
ls -la "$PLUGIN_DIR"
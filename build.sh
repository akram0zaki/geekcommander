#!/bin/bash

# Geek Commander Build Script for Linux/macOS
# This script helps build and install the application

set -e

RELEASE=false
INSTALL=false
CLEAN=false
TEST=false
HELP=false

show_help() {
    echo "Geek Commander Build Script"
    echo ""
    echo "Usage: ./build.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --release    Build optimized release version"
    echo "  --install    Install binary to system PATH"
    echo "  --clean      Clean build artifacts"
    echo "  --test       Run tests"
    echo "  --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./build.sh              # Debug build"
    echo "  ./build.sh --release    # Release build"
    echo "  ./build.sh --test       # Run tests"
    echo "  ./build.sh --release --install  # Build and install"
}

check_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

install_rust() {
    echo "Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "Rust installed successfully!"
}

build_project() {
    local is_release=$1
    
    echo "Building Geek Commander..."
    
    if [ "$is_release" = true ]; then
        echo "Building release version..."
        cargo build --release
        if [ $? -eq 0 ]; then
            echo "Build successful!"
            echo "Binary location: ./target/release/geekcommander"
        fi
    else
        echo "Building debug version..."
        cargo build
        if [ $? -eq 0 ]; then
            echo "Build successful!"
            echo "Binary location: ./target/debug/geekcommander"
        fi
    fi
    
    return $?
}

install_binary() {
    local is_release=$1
    local binary_path
    
    if [ "$is_release" = true ]; then
        binary_path="./target/release/geekcommander"
    else
        binary_path="./target/debug/geekcommander"
    fi
    
    if [ ! -f "$binary_path" ]; then
        echo "Binary not found at $binary_path. Please build first."
        return 1
    fi
    
    echo "Installing binary to system..."
    
    # Try to install to user's local bin directory first
    local local_bin="$HOME/.cargo/bin"
    if [ -d "$local_bin" ]; then
        cp "$binary_path" "$local_bin/geekcommander"
        echo "Installed to $local_bin/geekcommander"
        echo "You can now run 'geekcommander' from anywhere"
        return 0
    fi
    
    # Fallback: try system directory (requires sudo)
    if sudo cp "$binary_path" "/usr/local/bin/geekcommander"; then
        echo "Installed to /usr/local/bin/geekcommander"
        echo "You can now run 'geekcommander' from anywhere"
        return 0
    else
        echo "Failed to install to system directory"
        echo "Manual installation:"
        echo "  1. Copy $binary_path to a directory in your PATH"
        echo "  2. Or add the target directory to your PATH"
        return 1
    fi
}

clean_build() {
    echo "Cleaning build artifacts..."
    cargo clean
    if [ $? -eq 0 ]; then
        echo "Clean completed!"
    fi
}

run_tests() {
    echo "Running tests..."
    cargo test
    if [ $? -eq 0 ]; then
        echo "All tests passed!"
    else
        echo "Some tests failed!"
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE=true
            shift
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --test)
            TEST=true
            shift
            ;;
        --help)
            HELP=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main script logic
if [ "$HELP" = true ]; then
    show_help
    exit 0
fi

# Check if Rust is installed
if ! check_cargo; then
    install_rust
fi

# Perform requested actions
if [ "$CLEAN" = true ]; then
    clean_build
fi

if [ "$TEST" = true ]; then
    run_tests
fi

# Default action: build
if [ "$CLEAN" = false ] && [ "$TEST" = false ] && [ "$HELP" = false ]; then
    if build_project $RELEASE; then
        if [ "$INSTALL" = true ]; then
            install_binary $RELEASE
        fi
        
        echo ""
        echo "Next steps:"
        if [ "$RELEASE" = true ]; then
            echo "  Run: ./target/release/geekcommander"
        else
            echo "  Run: ./target/debug/geekcommander"
            echo "  Or:  cargo run"
        fi
        
        if [ "$INSTALL" = false ]; then
            echo "  Install: ./build.sh --release --install"
        fi
    fi
fi 
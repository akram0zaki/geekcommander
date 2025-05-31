# Quick Start Guide

This guide will help you get Geek Commander up and running quickly.

## Prerequisites

You need Rust installed on your system. If you don't have it:

### Windows
1. Download and install `rustup-init.exe` from https://www.rust-lang.org/tools/install
2. Follow the installer prompts (use default options)
3. Restart your terminal/PowerShell

### Linux/macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Building the Application

### Option 1: Use the Build Scripts (Recommended)

**Windows (PowerShell):**
```powershell
# Build debug version
.\build.ps1

# Build release version (optimized)
.\build.ps1 -Release

# Build and install to system PATH
.\build.ps1 -Release -Install

# Show help
.\build.ps1 -Help
```

**Linux/macOS:**
```bash
# Make script executable
chmod +x build.sh

# Build debug version
./build.sh

# Build release version (optimized)
./build.sh --release

# Build and install to system PATH
./build.sh --release --install

# Show help
./build.sh --help
```

### Option 2: Use Cargo Directly

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run directly
cargo run

# Install to system
cargo install --path .
```

## Running the Application

After building:

**Debug version:**
```bash
# Windows
.\target\debug\geekcommander.exe

# Linux/macOS
./target/debug/geekcommander
```

**Release version:**
```bash
# Windows
.\target\release\geekcommander.exe

# Linux/macOS
./target/release/geekcommander
```

**If installed to system:**
```bash
geekcommander
```

## Basic Usage

Once running:

- **↑/↓**: Move cursor up/down
- **Tab**: Switch between left and right panes
- **Enter**: Enter directory or view file
- **Backspace**: Go to parent directory
- **F1**: Show help
- **F5**: Copy files
- **F6**: Move/rename files
- **F7**: Create directory
- **F8**: Delete files
- **F10**: Quit

## Command Line Options

```bash
geekcommander [OPTIONS]

Options:
    --config <FILE>     Use custom config file
    --left <PATH>       Set left pane start directory
    --right <PATH>      Set right pane start directory
    --no-color          Force monochrome mode
    -h, --help          Show help information
    -V, --version       Show version information
```

## Examples

```bash
# Start with specific directories
geekcommander --left /home/user/documents --right /tmp

# Use custom config
geekcommander --config my-config.ini

# Start in monochrome mode
geekcommander --no-color
```

## Troubleshooting

**If Rust is not found:**
- Make sure you've installed Rust and restarted your terminal
- Try running: `rustup update`

**If build fails:**
- Make sure you have an internet connection (for downloading dependencies)
- Try: `cargo clean` then rebuild

**If the application doesn't start:**
- Make sure your terminal supports colors and function keys
- Try running with `--no-color` flag

## Configuration

The application creates a config file at:
- **Windows**: `%USERPROFILE%\.geekcommanderrc`
- **Linux/macOS**: `~/.geekcommanderrc`

You can customize keybindings, colors, and default paths by editing this file.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out the [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- Report issues or request features on the project repository 
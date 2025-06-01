# Geek Commander

A modern, cross-platform, dual-pane file manager inspired by Norton Commander, built with Rust, tui-rs, and crossterm.

![Application screen](images/screenshot.png)

## Overview

Geek Commander brings the classic Norton Commander experience to modern terminals, providing:

- **Dual-pane interface** with independent navigation
- **Keyboard-driven controls** (F1-F10 function keys, arrows, Tab)
- **File operations**: copy, move, delete, rename, create directory
- **Archive support**: browse and extract ZIP, TAR, TGZ files
- **Cross-platform**: Works on Windows, Linux, and macOS
- **Configurable**: Customizable keybindings and colors
- **Fast and lightweight**: Built with Rust for performance and safety

## Installation

### Prerequisites

You'll need Rust installed on your system. If you don't have Rust installed:

#### Windows:
1. Download and run 'rustup' - the official Rust toolchain installer and version manager - from https://www.rust-lang.org/tools/install
2. Follow the installer prompts (use default options)
3. Restart your terminal/PowerShell

Alternatively you can download the Windows build [geekcommander.exe](target/release/geekcommander.exe) from the repository. However I do encourage you to create the build yourself following the instructions in this guide.

#### Linux/macOS:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Building from Source

1. **Obtain the repository**

   You have two main options to get the code without needing a GitHub account:

   **A. Clone via Git (no credentials required for public repos)**  
   - **Windows (PowerShell or Git Bash)**  
     1. Install Git for Windows if you don’t already have it:  
        - Download from https://git-scm.com/download/win and run the installer (choose “Use Git from the Windows Command Prompt” or “Git Bash only,” whichever you prefer).  
     2. Open **Git Bash** (or PowerShell, if you added Git to your PATH).  
     3. Navigate to the folder where you want to store the project, for example:  
        ```powershell
        cd C:\Users\<YourName>\Projects
        ```  
     4. Run the clone command (this does not require a GitHub login if the repo is public):  
        ```bash
        git clone https://github.com/akram0zaki/geekcommander.git
        ```  
     5. Change into the new directory:  
        ```bash
        cd geekcommander
        ```  

   - **Linux (any distro with Git installed)**  
     1. Make sure Git is installed. On Ubuntu/Debian you can run:  
        ```bash
        sudo apt update
        sudo apt install git
        ```  
        On Fedora:  
        ```bash
        sudo dnf install git
        ```  
     2. Open your terminal and go to the folder where you want to clone:  
        ```bash
        cd ~/projects
        ```  
     3. Clone the repo (no login needed for a public repository):  
        ```bash
        git clone https://github.com/akram0zaki/geekcommander.git
        ```  
     4. Enter the directory:  
        ```bash
        cd geekcommander
        ```  

   **B. Download the ZIP archive (no Git needed at all)**  
   1. Open a web browser and go to:  
      ```
      https://github.com/akram0zaki/geekcommander
      ```  
   2. Click the green **Code** button (top right) → **Download ZIP**.  
   3. Extract the downloaded `geekcommander-main.zip` (on Windows: right-click → “Extract All…”; on Linux/macOS: `unzip geekcommander-main.zip`).  
   4. Rename or move the extracted folder so it’s easy to find. For example:  
      - On Windows Explorer, extract to `C:\Users\<YourName>\Projects\geekcommander`  
      - On Linux:  
        ```bash
        mkdir -p ~/projects
        unzip ~/Downloads/geekcommander-main.zip -d ~/projects/
        mv ~/projects/geekcommander-main ~/projects/geekcommander
        cd ~/projects/geekcommander
        ```  

2. **Open a terminal in the project directory**  
   - If you cloned using Git Bash/PowerShell (Windows) or a regular terminal (Linux), make sure your current working directory is the `geekcommander` folder.  
   - If you extracted from ZIP, open a terminal (or PowerShell) and `cd` into the folder you unzipped (e.g., `cd geekcommander`).

3. Build the project:

**A. Option 1: Use the Build Scripts (Recommended)** 

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

**B. Option 2: Use Cargo Directly** 

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

4. Run the application:

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

## Usage

### Quick Start

1. Launch the application:
   ```bash
   geekcommander
   ```

2. Navigate using:
   - **↑/↓**: Move cursor up/down
   - **Tab**: Switch between left and right panes
   - **Enter**: Enter directory or view file
   - **Backspace**: Go to parent directory

3. Use function keys for file operations:
   - **F1**: Help
   - **F5**: Copy
   - **F6**: Move/Rename
   - **F7**: Create directory
   - **F8**: Delete
   - **F10**: Quit

### Key Bindings

| Key | Action |
|-----|--------|
| **F1** | Show help screen |
| **F3** | View file |
| **F4** | Edit file |
| **F5** | Copy selected files |
| **F6** | Move/rename files |
| **F7** | Create new directory |
| **F8** | Delete selected files |
| **F10** | Quit application |
| **Tab** | Switch active pane |
| **↑/↓** | Move cursor |
| **Enter** | Enter directory/open file |
| **Backspace** | Go to parent directory |
| **Insert** | Select/unselect file |
| **Ctrl+A** | Select all files |

### Command Line Options

```
geekcommander [OPTIONS]

Options:
    --config <FILE>     Use custom config file
    --left <PATH>       Set left pane start directory
    --right <PATH>      Set right pane start directory
    --no-color          Force monochrome mode
    -h, --help          Show help information
    -V, --version       Show version information
```

**Examples**

```bash
# Start with specific directories
geekcommander --left /home/user/documents --right /tmp

# Use custom config
geekcommander --config my-config.ini

# Start in monochrome mode
geekcommander --no-color
```

## Configuration

Geek Commander creates a configuration file at:
- **Linux/macOS**: `~/.geekcommanderrc`
- **Windows**: `%USERPROFILE%\.geekcommanderrc`

### Sample Configuration

```ini
[Keybindings]
Help=F1
Copy=F5
Move=Shift+F5
Delete=F8
Rename=F6
NewDir=F7
Quit=F10
View=F3
Edit=F4
Select=Insert
SelectAll=Ctrl+A
Wildcard=*
Reload=Ctrl+R

[Colors]
ActivePaneBorder=cyan
InactivePaneBorder=white
SelectedItem=yellow
StatusBar=white
DirectoryFg=blue
FileFg=white

[Panels]
Left=~
Right=~

[Logging]
Level=INFO
File=~/.geekcommander.log

[General]
ShowHidden=false
FollowSymlinks=true
```

### Customization

- **Keybindings**: Modify the `[Keybindings]` section to change key mappings
- **Colors**: Available colors: black, red, green, yellow, blue, magenta, cyan, white
- **Start Paths**: Set default directories for left and right panes
- **Hidden Files**: Toggle visibility of hidden files (files starting with `.`)

## Features

### Dual-Pane File Management
- Independent navigation in left and right panes
- Visual indication of active pane
- Path display in pane headers
- File size and type indicators

### File Operations
- **Copy (F5)**: Copy files between panes with progress indication
- **Move (F6)**: Move files or rename single files
- **Delete (F8)**: Delete files and directories with confirmation
- **Create Directory (F7)**: Create new directories
- **File Selection**: Multi-select with Insert key

### Archive Support
- **Browse Archives**: Navigate inside ZIP, TAR, and TGZ files
- **Extract Files**: Extract archive contents to filesystem
- **Create Archives**: Add files to ZIP archives

### Cross-Platform Features
- **Windows**: Native Windows Terminal and PowerShell support
- **Linux**: Works with all major terminal emulators
- **macOS**: Full Terminal.app and iTerm2 compatibility

## Development

### Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # Configuration handling
├── core.rs          # File operations and pane management
├── ui.rs            # Terminal UI with tui-rs
├── archive.rs       # Archive file support
├── platform.rs      # Cross-platform utilities
└── error.rs         # Error types and handling
```

### Building for Different Targets

```bash
# Windows (from Linux/macOS)
cargo build --target x86_64-pc-windows-gnu

# Linux (static binary)
cargo build --target x86_64-unknown-linux-musl

# macOS
cargo build --target x86_64-apple-darwin
```

### Running Tests

```bash
# Run unit tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Configuration

The application creates a config file at:
- **Windows**: `%USERPROFILE%\.geekcommanderrc`
- **Linux/macOS**: `~/.geekcommanderrc`

You can customize keybindings, colors, and default paths by editing this file.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out the [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- Report issues or request features on the project repository 

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the original Norton Commander
- Built with [tui-rs](https://github.com/fdehau/tui-rs) and [crossterm](https://github.com/crossterm-rs/crossterm)
- Thanks to the Rust community for excellent libraries and tools

## Troubleshooting

### Common Issues

**If Rust is not found:**
- Make sure you've installed Rust and restarted your terminal
- Try running: `rustup update`

**If build fails:**
- Make sure you have an internet connection (for downloading dependencies)
- Try: `cargo clean` then rebuild

**If the application doesn't start:**
- Make sure your terminal supports colors and function keys
- Try running with `--no-color` flag

**Terminal Size Too Small:**
```
Error: Terminal too small. Minimum size: 80x24
```
Solution: Resize your terminal window or use `--no-color` flag.

**Permission Denied:**
```
Error: Permission denied accessing directory
```
Solution: Check file permissions or run with appropriate privileges.

**Config File Errors:**
```
Warning: Invalid keybinding 'F99' - using default
```
Solution: Check your `~/.geekcommanderrc` file for invalid entries.

### Logging

Debug information is logged to:
- **Linux/macOS**: `~/.geekcommander.log`
- **Windows**: `%USERPROFILE%\.geekcommander.log`

Set log level in config file:
```ini
[Logging]
Level=DEBUG  # Options: ERROR, WARN, INFO, DEBUG, TRACE
```

---

For more information, visit the [project repository](https://github.com/akram0zaki/geekcommander) or open an issue. 
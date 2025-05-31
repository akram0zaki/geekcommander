# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned Features
- File viewer (F3) with syntax highlighting
- Internal text editor (F4) or external editor integration
- Plugin system architecture
- Network protocol support (SFTP, FTP)
- Advanced archive formats (RAR, 7z)
- Search functionality within directories
- Bookmark system for frequently accessed directories
- File comparison and synchronization tools
- Themes and advanced color customization

## [1.0.5] - 2025-05-31

### Added
- **3-Column Norton Commander Layout**: Each pane now displays files in authentic Norton Commander style with Name, Size, and Date columns
- **Visible Column Borders**: Table widget implementation with proper column separation for enhanced readability
- **Date Formatting**: File modification times displayed in "mm-dd-yy hh:mm" format matching Norton Commander conventions
- **Responsive Column Widths**: Name column gets 70% of space, Size and Date columns get 15% each for optimal balance
- **Right-Aligned Data Columns**: Size and date columns are right-aligned for better readability and professional appearance

### Enhanced
- **File Display System**: Replaced List widget with Table widget for better column organization and visual structure
- **Icon Integration**: File type icons properly aligned within the Name column
- **Size Formatting**: Right-aligned file sizes with consistent formatting for easy comparison
- **Header Row**: Yellow column headers with proper alignment matching their respective column content
- **Space Utilization**: Optimized column proportions to maximize space for file names while keeping data columns compact

### Technical Improvements
- **Table State Management**: Proper cursor navigation and highlighting within the table structure
- **Platform Date Formatting**: Cross-platform date/time conversion using chrono library
- **Dynamic Column Sizing**: Responsive column width calculations based on terminal size
- **Memory Efficiency**: TUI framework handles table rendering optimization automatically
- **Code Organization**: Clean separation between display logic and data structures

### Testing
- **3-Column Test Suite**: New tests verify file entry properties, date formatting, and column data availability
- **Date Formatting Tests**: Comprehensive validation of time conversion and format consistency
- **Display Integration Tests**: Ensure all file types display correctly in the new column layout

## [1.0.4] - 2025-05-31

### Added
- **Comprehensive Test Coverage**: Added 10 new unit tests specifically for navigation and scrolling functionality
- **Navigation Test Suite**: Complete test coverage for all navigation methods (cursor_up, cursor_down, page_up, page_down, cursor_home, cursor_end)
- **Edge Case Testing**: Robust tests for empty directories, invalid cursor positions, and large directories (100+ files)
- **Boundary Testing**: Thorough verification of bounds checking and mathematical calculations
- **Regression Prevention**: Tests to ensure scroll_offset remains unused and corruption patterns don't return

### Improved
- **Test Quality**: All navigation tests use isolated temporary directories with predictable file sets
- **Documentation**: Comprehensive test coverage report documenting all navigation scenarios
- **Future-Proofing**: Regression detection for any changes that might break navigation functionality
- **Performance Validation**: Large directory tests establish performance baselines

### Technical
- **40 Total Tests**: All passing, including 12 new navigation-focused tests
- **Mathematical Verification**: Page navigation calculations verified with multiple viewport sizes
- **Error Scenario Coverage**: Invalid positions, empty directories, and edge cases thoroughly tested

## [1.0.3] - 2025-05-31

### Fixed
- **Scrolling Corruption**: Fixed visual corruption and messy display when scrolling through long directory listings
- **Framework Integration**: Resolved conflict with TUI framework by removing manual viewport management
- **Rendering Quality**: Eliminated visual artifacts caused by manual cursor overlays and entry slicing

### Technical Improvements
- **Simplified Scrolling**: Let TUI framework handle scrolling automatically instead of manual management
- **Clean Rendering**: Removed complex manual scroll offset calculations that caused corruption
- **Framework Compliance**: Now properly uses TUI List widget's built-in scrolling capabilities
- **Code Simplification**: Removed unnecessary manual viewport slicing and cursor overlay rendering

### Performance
- **Better Memory Usage**: TUI framework handles rendering optimization internally
- **Smoother Navigation**: Automatic scrolling provides consistent user experience
- **Reduced Complexity**: Simplified codebase is more maintainable and reliable

## [1.0.2] - 2025-05-31

### Fixed
- **Scrolling Bug**: Fixed issue where cursor would disappear beyond visible area when navigating through long directory listings
- **Viewport Management**: Improved pane scrolling to automatically keep selected item visible
- **Navigation Performance**: Enhanced rendering to only display visible entries for better performance with large directories

### Technical Improvements
- Added proper scroll offset calculation methods to `PaneState`
- Implemented `cursor_up()`, `cursor_down()`, `page_up()`, `page_down()`, `cursor_home()`, `cursor_end()` methods
- Enhanced `render_pane()` function to handle viewport-based rendering
- Fixed bounds checking for all navigation operations

## [1.0.1] - 2025-05-31

### 🔧 Fixed
- **Tab Key Navigation**: Fixed Tab key behavior that was briefly switching panes and switching back
- **Directory Navigation**: Fixed Enter key not working on directories - now properly enters directories and navigates up with ".." entries
- **Key Handling Priority**: Reorganized key handling to prioritize core navigation keys (Tab, arrows, Enter, Backspace) before custom keybindings

### 🎨 Enhanced
- **Norton Commander Classic Theme**: Updated color scheme to match original Norton Commander
  - Blue background for main interface
  - Cyan text on blue background
  - White text for directories with bold formatting
  - Cyan text for files
  - Black background with white text for selected items
  - Cyan highlight for current cursor position
- **Visual Consistency**: Applied blue theme throughout the entire interface including title bar, status bar, and panes

### 🧪 Testing
- All 25 unit tests continue to pass
- Configuration tests updated to reflect new color scheme defaults

## [1.0.0] - 2025-05-31

### Added

#### Core Features
- **Dual-pane file manager** interface inspired by Norton Commander
- **Cross-platform support** for Windows, Linux, and macOS
- **Terminal-based UI** using tui-rs and crossterm for modern terminal compatibility
- **Keyboard-driven navigation** with Norton Commander-style keybindings

#### File Operations
- **File copying (F5)** with progress indication and confirmation dialogs
- **File moving/renaming (F6)** for single files and batch operations
- **File deletion (F8)** with confirmation prompts
- **Directory creation (F7)** with interactive input
- **Multi-file selection** using Insert key and Ctrl+A for select all
- **Directory navigation** with Enter, Backspace, and arrow keys

#### Archive Support
- **ZIP archive browsing** - navigate inside ZIP files as virtual directories
- **TAR archive support** - browse TAR, TAR.GZ, and TGZ files
- **Archive extraction** - extract files from archives to filesystem
- **Archive creation** - add files to ZIP archives (basic support)

#### User Interface
- **Active pane highlighting** with configurable border colors
- **File type indicators** - [D] for directories, [F] for files, [A] for archives
- **File size display** with human-readable formatting (KB, MB, GB)
- **Status bar** showing current paths, free disk space, and key hints
- **Help screen (F1)** with comprehensive keybinding reference
- **Interactive dialogs** for confirmations, input, and error messages

#### Configuration System
- **INI-based configuration** file (`
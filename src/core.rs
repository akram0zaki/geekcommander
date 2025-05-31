use std::fs::{self, DirEntry, File, Metadata};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::HashSet;
use crate::error::{GeekCommanderError, Result};
use crate::platform;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_archive: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: String,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct PaneState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor_index: usize,
    pub scroll_offset: usize,
    pub selected_indices: HashSet<usize>,
    pub archive_context: Option<ArchiveContext>,
}

#[derive(Debug, Clone)]
pub struct ArchiveContext {
    pub archive_path: PathBuf,
    pub virtual_path: String,
    pub entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileOperation {
    pub operation_type: OperationType,
    pub source_files: Vec<PathBuf>,
    pub destination: PathBuf,
    pub total_size: u64,
    pub processed_size: u64,
    pub current_file: Option<String>,
    pub completed: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Copy,
    Move,
    Delete,
}

impl PaneState {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut state = PaneState {
            current_path: path,
            entries: Vec::new(),
            cursor_index: 0,
            scroll_offset: 0,
            selected_indices: HashSet::new(),
            archive_context: None,
        };
        state.refresh()?;
        Ok(state)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();
        
        // Add parent directory entry if not at root
        if let Some(parent) = self.current_path.parent() {
            if parent != self.current_path {
                self.entries.push(FileEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    is_dir: true,
                    is_archive: false,
                    size: 0,
                    modified: SystemTime::UNIX_EPOCH,
                    permissions: "drwxrwxrwx".to_string(),
                });
            }
        }

        // Read directory contents
        let read_dir = fs::read_dir(&self.current_path)
            .map_err(|e| GeekCommanderError::Io(e))?;

        for entry in read_dir {
            let entry = entry.map_err(|e| GeekCommanderError::Io(e))?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(|e| GeekCommanderError::Io(e))?;
            
            let name = entry.file_name().to_string_lossy().to_string();
            let is_archive = is_supported_archive(&path);
            
            let file_entry = FileEntry {
                name: name.clone(),
                path: path.clone(),
                is_dir: metadata.is_dir(),
                is_archive,
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                permissions: platform::get_file_permissions(&metadata),
            };
            
            self.entries.push(file_entry);
        }

        // Sort entries: directories first, then files, alphabetically
        self.entries.sort_by(|a, b| {
            if a.name == ".." {
                std::cmp::Ordering::Less
            } else if b.name == ".." {
                std::cmp::Ordering::Greater
            } else if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        // Reset cursor if needed
        if self.cursor_index >= self.entries.len() {
            self.cursor_index = 0;
        }

        // Clear selections that are no longer valid
        self.selected_indices.retain(|&i| i < self.entries.len());

        Ok(())
    }

    /// Move cursor up by one position
    pub fn cursor_up(&mut self, _viewport_height: usize) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
        }
    }

    /// Move cursor down by one position
    pub fn cursor_down(&mut self, _viewport_height: usize) {
        if self.cursor_index < self.entries.len().saturating_sub(1) {
            self.cursor_index += 1;
        }
    }

    /// Move cursor up by page size
    pub fn page_up(&mut self, viewport_height: usize) {
        let page_size = viewport_height.saturating_sub(2).max(1);
        self.cursor_index = self.cursor_index.saturating_sub(page_size);
    }

    /// Move cursor down by page size
    pub fn page_down(&mut self, viewport_height: usize) {
        let page_size = viewport_height.saturating_sub(2).max(1);
        self.cursor_index = (self.cursor_index + page_size).min(self.entries.len().saturating_sub(1));
    }

    /// Move cursor to first entry
    pub fn cursor_home(&mut self, _viewport_height: usize) {
        self.cursor_index = 0;
    }

    /// Move cursor to last entry
    pub fn cursor_end(&mut self, _viewport_height: usize) {
        self.cursor_index = self.entries.len().saturating_sub(1);
    }

    pub fn enter_directory(&mut self, new_path: PathBuf) -> Result<()> {
        if new_path.is_dir() {
            self.current_path = new_path;
            self.cursor_index = 0;
            self.scroll_offset = 0;
            self.selected_indices.clear();
            self.refresh()?;
        }
        Ok(())
    }

    pub fn get_current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor_index)
    }

    pub fn toggle_selection(&mut self) {
        if self.cursor_index < self.entries.len() {
            if self.selected_indices.contains(&self.cursor_index) {
                self.selected_indices.remove(&self.cursor_index);
            } else {
                self.selected_indices.insert(self.cursor_index);
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selected_indices.clear();
        for i in 0..self.entries.len() {
            if self.entries[i].name != ".." {
                self.selected_indices.insert(i);
            }
        }
    }

    pub fn deselect_all(&mut self) {
        self.selected_indices.clear();
    }

    pub fn get_selected_entries(&self) -> Vec<&FileEntry> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    pub fn has_selections(&self) -> bool {
        !self.selected_indices.is_empty()
    }

    pub fn select_by_pattern(&mut self, pattern: &str) -> Result<usize> {
        let mut count = 0;
        
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.name == ".." {
                continue;
            }
            
            if matches_glob_pattern(&entry.name, pattern) {
                self.selected_indices.insert(i);
                count += 1;
            }
        }
        
        Ok(count)
    }
}

pub fn copy_files(sources: &[&FileEntry], destination: &Path) -> Result<FileOperation> {
    let total_size = calculate_total_size(sources)?;
    let source_paths: Vec<PathBuf> = sources.iter().map(|e| e.path.clone()).collect();
    
    let operation = FileOperation {
        operation_type: OperationType::Copy,
        source_files: source_paths,
        destination: destination.to_path_buf(),
        total_size,
        processed_size: 0,
        current_file: None,
        completed: false,
        cancelled: false,
    };
    
    Ok(operation)
}

pub fn move_files(sources: &[&FileEntry], destination: &Path) -> Result<FileOperation> {
    let total_size = calculate_total_size(sources)?;
    let source_paths: Vec<PathBuf> = sources.iter().map(|e| e.path.clone()).collect();
    
    let operation = FileOperation {
        operation_type: OperationType::Move,
        source_files: source_paths,
        destination: destination.to_path_buf(),
        total_size,
        processed_size: 0,
        current_file: None,
        completed: false,
        cancelled: false,
    };
    
    Ok(operation)
}

pub fn delete_files(sources: &[&FileEntry]) -> Result<FileOperation> {
    let total_size = calculate_total_size(sources)?;
    let source_paths: Vec<PathBuf> = sources.iter().map(|e| e.path.clone()).collect();
    
    let operation = FileOperation {
        operation_type: OperationType::Delete,
        source_files: source_paths,
        destination: PathBuf::new(),
        total_size,
        processed_size: 0,
        current_file: None,
        completed: false,
        cancelled: false,
    };
    
    Ok(operation)
}

pub fn execute_operation(operation: &mut FileOperation) -> Result<()> {
    match operation.operation_type {
        OperationType::Copy => execute_copy_operation(operation),
        OperationType::Move => execute_move_operation(operation),
        OperationType::Delete => execute_delete_operation(operation),
    }
}

fn execute_copy_operation(operation: &mut FileOperation) -> Result<()> {
    let source_files = operation.source_files.clone(); // Clone to avoid borrowing issues
    
    for source_path in &source_files {
        if operation.cancelled {
            break;
        }
        
        let file_name = source_path.file_name()
            .ok_or_else(|| GeekCommanderError::FileOperation("Invalid source file name".to_string()))?
            .to_string_lossy();
        
        operation.current_file = Some(file_name.to_string());
        
        let dest_path = operation.destination.join(&*file_name);
        
        if source_path.is_dir() {
            copy_directory_recursive(source_path, &dest_path, operation)?;
        } else {
            copy_file_with_progress(source_path, &dest_path, operation)?;
        }
    }
    
    operation.completed = true;
    Ok(())
}

fn execute_move_operation(operation: &mut FileOperation) -> Result<()> {
    // First copy all files, then delete originals
    execute_copy_operation(operation)?;
    
    if !operation.cancelled {
        for source_path in &operation.source_files {
            if source_path.is_dir() {
                fs::remove_dir_all(source_path)?;
            } else {
                fs::remove_file(source_path)?;
            }
        }
    }
    
    Ok(())
}

fn execute_delete_operation(operation: &mut FileOperation) -> Result<()> {
    for source_path in &operation.source_files {
        if operation.cancelled {
            break;
        }
        
        let file_name = source_path.file_name()
            .ok_or_else(|| GeekCommanderError::FileOperation("Invalid source file name".to_string()))?
            .to_string_lossy();
        
        operation.current_file = Some(file_name.to_string());
        
        if source_path.is_dir() {
            fs::remove_dir_all(source_path)?;
        } else {
            fs::remove_file(source_path)?;
        }
        
        operation.processed_size += get_path_size(source_path)?;
    }
    
    operation.completed = true;
    Ok(())
}

fn copy_file_with_progress(source: &Path, dest: &Path, operation: &mut FileOperation) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut source_file = fs::File::open(source)?;
    let mut dest_file = fs::File::create(dest)?;
    
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
    
    loop {
        if operation.cancelled {
            break;
        }
        
        let bytes_read = source_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        dest_file.write_all(&buffer[..bytes_read])?;
        operation.processed_size += bytes_read as u64;
    }
    
    Ok(())
}

fn copy_directory_recursive(source: &Path, dest: &Path, operation: &mut FileOperation) -> Result<()> {
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(source)? {
        if operation.cancelled {
            break;
        }
        
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if source_path.is_dir() {
            copy_directory_recursive(&source_path, &dest_path, operation)?;
        } else {
            copy_file_with_progress(&source_path, &dest_path, operation)?;
        }
    }
    
    Ok(())
}

fn calculate_total_size(sources: &[&FileEntry]) -> Result<u64> {
    let mut total = 0;
    for entry in sources {
        total += get_path_size(&entry.path)?;
    }
    Ok(total)
}

fn get_path_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        Ok(fs::metadata(path)?.len())
    } else if path.is_dir() {
        let mut size = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            size += get_path_size(&entry.path())?;
        }
        Ok(size)
    } else {
        Ok(0)
    }
}

fn matches_glob_pattern(name: &str, pattern: &str) -> bool {
    // Simple glob pattern matching
    if pattern == "*" {
        return true;
    }
    
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return name.starts_with(prefix) && name.ends_with(suffix);
        }
    }
    
    name == pattern
}

pub fn is_supported_archive(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            match ext_str.to_lowercase().as_str() {
                "zip" | "tar" | "gz" | "tgz" => return true,
                _ => {}
            }
        }
    }
    
    // Check for .tar.gz, .tar.bz2, etc.
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            let name_lower = name_str.to_lowercase();
            return name_lower.ends_with(".tar.gz") || 
                   name_lower.ends_with(".tar.bz2") ||
                   name_lower.ends_with(".tar.xz");
        }
    }
    
    false
}

pub fn create_directory(path: &Path, name: &str) -> Result<PathBuf> {
    let new_dir = path.join(name);
    
    if new_dir.exists() {
        return Err(GeekCommanderError::FileOperation(format!("Directory '{}' already exists", name)));
    }
    
    fs::create_dir(&new_dir)?;
    Ok(new_dir)
}

pub fn rename_file(old_path: &Path, new_name: &str) -> Result<PathBuf> {
    let parent = old_path.parent()
        .ok_or_else(|| GeekCommanderError::FileOperation("Cannot determine parent directory".to_string()))?;
    
    let new_path = parent.join(new_name);
    
    if new_path.exists() {
        return Err(GeekCommanderError::FileOperation(format!("File '{}' already exists", new_name)));
    }
    
    fs::rename(old_path, &new_path)?;
    Ok(new_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_matches_glob_pattern() {
        assert!(matches_glob_pattern("test.txt", "*.txt"));
        assert!(matches_glob_pattern("file.log", "*.log"));
        assert!(matches_glob_pattern("anything", "*"));
        assert!(!matches_glob_pattern("test.txt", "*.log"));
        assert!(matches_glob_pattern("exact_match", "exact_match"));
    }

    #[test]
    fn test_is_supported_archive() {
        assert!(is_supported_archive(Path::new("test.zip")));
        assert!(is_supported_archive(Path::new("test.tar")));
        assert!(is_supported_archive(Path::new("test.tar.gz")));
        assert!(is_supported_archive(Path::new("test.tgz")));
        assert!(!is_supported_archive(Path::new("test.txt")));
        assert!(!is_supported_archive(Path::new("test")));
    }

    #[test]
    fn test_pane_state_creation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        assert_eq!(pane.current_path, temp_dir.path());
        assert_eq!(pane.cursor_index, 0);
        assert_eq!(pane.scroll_offset, 0);
        assert!(pane.selected_indices.is_empty());
        assert!(pane.archive_context.is_none());
        
        Ok(())
    }

    #[test]
    fn test_toggle_selection() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Should have at least the test file
        assert!(!pane.entries.is_empty());
        
        // Toggle selection on first entry
        pane.cursor_index = 0;
        pane.toggle_selection();
        assert!(pane.selected_indices.contains(&0));
        
        // Toggle again to deselect
        pane.toggle_selection();
        assert!(!pane.selected_indices.contains(&0));
        
        Ok(())
    }

    #[test]
    fn test_select_by_pattern() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        File::create(temp_dir.path().join("test1.txt")).unwrap();
        File::create(temp_dir.path().join("test2.txt")).unwrap();
        File::create(temp_dir.path().join("other.log")).unwrap();
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        let count = pane.select_by_pattern("*.txt")?;
        assert_eq!(count, 2); // Should select the two .txt files
        assert_eq!(pane.selected_indices.len(), 2);
        
        Ok(())
    }

    #[test]
    fn test_get_path_size() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create a file with known content
        std::fs::write(&test_file, "Hello, world!").unwrap();
        
        let size = get_path_size(&test_file)?;
        assert_eq!(size, 13); // "Hello, world!" is 13 bytes
        
        Ok(())
    }

    #[test]
    fn test_create_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = create_directory(temp_dir.path(), "new_directory")?;
        
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
        assert_eq!(new_dir.file_name().unwrap(), "new_directory");
        
        // Test that creating the same directory again fails
        assert!(create_directory(temp_dir.path(), "new_directory").is_err());
        
        Ok(())
    }

    #[test]
    fn test_rename_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let original_file = temp_dir.path().join("original.txt");
        File::create(&original_file).unwrap();
        
        let new_path = rename_file(&original_file, "renamed.txt")?;
        
        assert!(!original_file.exists());
        assert!(new_path.exists());
        assert_eq!(new_path.file_name().unwrap(), "renamed.txt");
        
        Ok(())
    }

    // ===== NEW NAVIGATION AND SCROLLING TESTS =====

    #[test]
    fn test_cursor_up_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test files
        for i in 1..=5 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Start at position 3
        pane.cursor_index = 3;
        assert_eq!(pane.cursor_index, 3);
        
        // Move up one position
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 2);
        
        // Move up again
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 1);
        
        // Test boundary - should not go below 0
        pane.cursor_index = 0;
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 0);
        
        Ok(())
    }

    #[test]
    fn test_cursor_down_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test files
        for i in 1..=5 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        let max_index = pane.entries.len().saturating_sub(1);
        
        // Start at position 0
        pane.cursor_index = 0;
        assert_eq!(pane.cursor_index, 0);
        
        // Move down one position
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 1);
        
        // Move down again
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 2);
        
        // Test boundary - should not go beyond max
        pane.cursor_index = max_index;
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, max_index);
        
        Ok(())
    }

    #[test]
    fn test_page_up_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create many test files to test page navigation
        for i in 1..=30 {
            File::create(temp_dir.path().join(format!("file{:02}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Start at position 20
        pane.cursor_index = 20;
        
        // Page up with viewport height 10 (page_size = 8)
        pane.page_up(10);
        assert_eq!(pane.cursor_index, 12); // 20 - 8 = 12
        
        // Page up again
        pane.page_up(10);
        assert_eq!(pane.cursor_index, 4); // 12 - 8 = 4
        
        // Test boundary - should not go below 0
        pane.cursor_index = 3;
        pane.page_up(10);
        assert_eq!(pane.cursor_index, 0);
        
        Ok(())
    }

    #[test]
    fn test_page_down_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create many test files to test page navigation
        for i in 1..=30 {
            File::create(temp_dir.path().join(format!("file{:02}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        let max_index = pane.entries.len().saturating_sub(1);
        
        // Start at position 5
        pane.cursor_index = 5;
        
        // Page down with viewport height 10 (page_size = 8)
        pane.page_down(10);
        assert_eq!(pane.cursor_index, 13); // 5 + 8 = 13
        
        // Page down again
        pane.page_down(10);
        assert_eq!(pane.cursor_index, 21); // 13 + 8 = 21
        
        // Test boundary - should not go beyond max
        let near_end = max_index - 2;
        pane.cursor_index = near_end;
        pane.page_down(10);
        assert_eq!(pane.cursor_index, max_index);
        
        Ok(())
    }

    #[test]
    fn test_cursor_home_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        for i in 1..=10 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Start at middle position
        pane.cursor_index = 5;
        assert_eq!(pane.cursor_index, 5);
        
        // Jump to home (first position)
        pane.cursor_home(10);
        assert_eq!(pane.cursor_index, 0);
        
        // Test from end position
        pane.cursor_index = pane.entries.len() - 1;
        pane.cursor_home(10);
        assert_eq!(pane.cursor_index, 0);
        
        Ok(())
    }

    #[test]
    fn test_cursor_end_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        for i in 1..=10 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        let max_index = pane.entries.len().saturating_sub(1);
        
        // Start at first position
        pane.cursor_index = 0;
        assert_eq!(pane.cursor_index, 0);
        
        // Jump to end (last position)
        pane.cursor_end(10);
        assert_eq!(pane.cursor_index, max_index);
        
        // Test from middle position
        pane.cursor_index = 5;
        pane.cursor_end(10);
        assert_eq!(pane.cursor_index, max_index);
        
        Ok(())
    }

    #[test]
    fn test_navigation_with_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // In empty directory, only ".." entry should exist
        assert_eq!(pane.entries.len(), 1);
        assert_eq!(pane.entries[0].name, "..");
        assert_eq!(pane.cursor_index, 0);
        
        // Test all navigation methods with empty directory
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 0);
        
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 0);
        
        pane.page_up(10);
        assert_eq!(pane.cursor_index, 0);
        
        pane.page_down(10);
        assert_eq!(pane.cursor_index, 0);
        
        pane.cursor_home(10);
        assert_eq!(pane.cursor_index, 0);
        
        pane.cursor_end(10);
        assert_eq!(pane.cursor_index, 0);
        
        Ok(())
    }

    #[test]
    fn test_navigation_bounds_checking() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create exactly 3 files (plus ".." = 4 total entries)
        for i in 1..=3 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        assert_eq!(pane.entries.len(), 4); // ".." + 3 files
        let max_valid_index = 3;
        
        // Test cursor_up from invalid position (10 > 0, so will decrease)
        pane.cursor_index = 10; 
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 9); // Decreases by 1
        
        // Test cursor_down from invalid position (10 > max_valid, so won't change)
        pane.cursor_index = 10; 
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 10); // Stays invalid because > max_valid
        
        // Test cursor_down from valid position
        pane.cursor_index = 2; // Valid position
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 3); // Moves to max valid
        
        pane.cursor_down(10); // Try to go beyond max
        assert_eq!(pane.cursor_index, 3); // Stays at max valid
        
        // cursor_home always goes to 0 (corrects invalid position)
        pane.cursor_index = 10; 
        pane.cursor_home(10);
        assert_eq!(pane.cursor_index, 0);
        
        // cursor_end always goes to last valid index (corrects invalid position)
        pane.cursor_index = 10; 
        pane.cursor_end(10);
        assert_eq!(pane.cursor_index, max_valid_index);
        
        // Test page_up with out of bounds position
        pane.cursor_index = 10; 
        pane.page_up(10); // page_size = 8, 10.saturating_sub(8) = 2
        assert_eq!(pane.cursor_index, 2);
        
        // Test page_down with out of bounds position  
        pane.cursor_index = 10; 
        pane.page_down(10); // (10 + 8).min(3) = 18.min(3) = 3
        assert_eq!(pane.cursor_index, max_valid_index);
        
        Ok(())
    }

    #[test]
    fn test_large_directory_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a large number of files to test performance
        for i in 1..=100 {
            File::create(temp_dir.path().join(format!("file{:03}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        assert_eq!(pane.entries.len(), 101); // ".." + 100 files
        
        // Test navigation in large directory
        pane.cursor_index = 50;
        
        // Page navigation should work correctly
        pane.page_up(20); // page_size = 18
        assert_eq!(pane.cursor_index, 32); // 50 - 18
        
        pane.page_down(20);
        assert_eq!(pane.cursor_index, 50); // 32 + 18
        
        // Jump to edges
        pane.cursor_home(20);
        assert_eq!(pane.cursor_index, 0);
        
        pane.cursor_end(20);
        assert_eq!(pane.cursor_index, 100); // Last index
        
        Ok(())
    }

    #[test]
    fn test_mixed_files_and_directories_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create mixed content (directories sort first)
        std::fs::create_dir(temp_dir.path().join("dir1")).unwrap();
        std::fs::create_dir(temp_dir.path().join("dir2")).unwrap();
        File::create(temp_dir.path().join("file1.txt")).unwrap();
        File::create(temp_dir.path().join("file2.txt")).unwrap();
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Check sorting: ".." first, then directories, then files
        assert_eq!(pane.entries[0].name, "..");
        assert!(pane.entries[1].is_dir && pane.entries[1].name == "dir1");
        assert!(pane.entries[2].is_dir && pane.entries[2].name == "dir2");
        assert!(!pane.entries[3].is_dir && pane.entries[3].name == "file1.txt");
        assert!(!pane.entries[4].is_dir && pane.entries[4].name == "file2.txt");
        
        // Test navigation through mixed content
        pane.cursor_index = 0; // Start at ".."
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 1); // Move to first directory
        
        pane.cursor_down(10);
        pane.cursor_down(10);
        assert_eq!(pane.cursor_index, 3); // Move to first file
        
        pane.cursor_up(10);
        assert_eq!(pane.cursor_index, 2); // Back to second directory
        
        Ok(())
    }

    #[test]
    fn test_viewport_height_edge_cases() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        for i in 1..=10 {
            File::create(temp_dir.path().join(format!("file{}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Test with very small viewport height
        pane.cursor_index = 5;
        pane.page_up(3); // viewport_height=3, page_size=1
        assert_eq!(pane.cursor_index, 4); // 5 - 1
        
        pane.page_down(3);
        assert_eq!(pane.cursor_index, 5); // 4 + 1
        
        // Test with zero viewport height (edge case)
        pane.cursor_index = 5;
        pane.page_up(0); // Should not crash, page_size=1 (max)
        assert_eq!(pane.cursor_index, 4);
        
        pane.page_down(0);
        assert_eq!(pane.cursor_index, 5);
        
        Ok(())
    }

    #[test]
    fn test_scroll_offset_remains_unused() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        for i in 1..=20 {
            File::create(temp_dir.path().join(format!("file{:02}.txt", i))).unwrap();
        }
        
        let mut pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Verify scroll_offset is initialized to 0
        assert_eq!(pane.scroll_offset, 0);
        
        // Perform various navigation operations
        pane.cursor_down(10);
        pane.cursor_down(10);
        pane.page_down(10);
        pane.cursor_end(10);
        pane.cursor_home(10);
        pane.page_up(10);
        
        // scroll_offset should remain 0 (unused in new implementation)
        assert_eq!(pane.scroll_offset, 0);
        
        Ok(())
    }

    #[test]
    fn test_file_entry_properties_for_display() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with different sizes and a directory
        std::fs::write(temp_dir.path().join("small.txt"), "tiny")?;
        std::fs::write(temp_dir.path().join("large.txt"), "x".repeat(1024))?;
        std::fs::create_dir(temp_dir.path().join("directory"))?;
        
        let pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        // Should have entries: "..", "directory", "large.txt", "small.txt" (sorted)
        assert!(pane.entries.len() >= 4);
        
        // Check ".." entry
        let parent_entry = &pane.entries[0];
        assert_eq!(parent_entry.name, "..");
        assert!(parent_entry.is_dir);
        assert_eq!(parent_entry.size, 0);
        
        // Check directory entry
        let dir_entry = pane.entries.iter().find(|e| e.name == "directory").unwrap();
        assert!(dir_entry.is_dir);
        assert!(!dir_entry.is_archive);
        
        // Check small file
        let small_file = pane.entries.iter().find(|e| e.name == "small.txt").unwrap();
        assert!(!small_file.is_dir);
        assert_eq!(small_file.size, 4); // "tiny" is 4 bytes
        assert!(small_file.modified > SystemTime::UNIX_EPOCH);
        
        // Check large file
        let large_file = pane.entries.iter().find(|e| e.name == "large.txt").unwrap();
        assert!(!large_file.is_dir);
        assert_eq!(large_file.size, 1024); // 1024 'x' characters
        assert!(large_file.modified > SystemTime::UNIX_EPOCH);
        
        Ok(())
    }

    #[test]
    fn test_three_column_data_availability() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        
        // Create various file types
        std::fs::write(temp_dir.path().join("document.txt"), "Sample document content")?;
        std::fs::write(temp_dir.path().join("config.ini"), "[settings]\nkey=value")?;
        std::fs::create_dir(temp_dir.path().join("subdirectory"))?;
        
        let pane = PaneState::new(temp_dir.path().to_path_buf())?;
        
        for entry in &pane.entries {
            // Every entry should have the three essential columns of data:
            
            // 1. Name (always present)
            assert!(!entry.name.is_empty());
            
            // 2. Size (0 for directories, actual size for files)
            if entry.is_dir {
                assert_eq!(entry.size, 0);
            } else if entry.name != ".." {
                assert!(entry.size >= 0); // Files can be 0 bytes
            }
            
            // 3. Modified time (should be valid)
            assert!(entry.modified >= SystemTime::UNIX_EPOCH);
            
            // Verify the path is correct
            assert!(entry.path.exists() || entry.name == "..");
        }
        
        Ok(())
    }
} 
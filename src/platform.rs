use std::path::{Path, PathBuf};
use crate::error::{GeekCommanderError, Result};

/// Get the available disk space for a given path
pub fn get_free_disk_space(path: &Path) -> Result<u64> {
    #[cfg(windows)]
    {
        use winapi::um::fileapi::GetDiskFreeSpaceExW;
        use winapi::shared::ntdef::ULARGE_INTEGER;
        use std::os::windows::ffi::OsStrExt;
        
        let wide_path: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        let mut free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        
        let result = unsafe {
            GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes,
                &mut total_bytes,
                std::ptr::null_mut(),
            )
        };
        
        if result != 0 {
            unsafe { Ok(*free_bytes.QuadPart() as u64) }
        } else {
            Err(GeekCommanderError::Io(std::io::Error::last_os_error()))
        }
    }
    
    #[cfg(not(windows))]
    {
        // Simplified fallback for non-Windows
        Ok(1024 * 1024 * 1024) // Return 1GB as fallback
    }
}

/// Normalize a path for the current platform
pub fn normalize_path(path: &Path) -> PathBuf {
    // Expand ~ to home directory
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    
    // Convert path separators to platform-specific
    #[cfg(windows)]
    {
        PathBuf::from(path.to_string_lossy().replace('/', "\\"))
    }
    
    #[cfg(not(windows))]
    {
        PathBuf::from(path.to_string_lossy().replace('\\', "/"))
    }
}

/// Check if a path is a root directory
pub fn is_root_path(path: &Path) -> bool {
    #[cfg(windows)]
    {
        // On Windows, check for drive roots like C:\
        path.parent().is_none() || 
        path.to_string_lossy().ends_with(":\\") ||
        path.to_string_lossy().ends_with(":/")
    }
    
    #[cfg(not(windows))]
    {
        // On Unix, root is /
        path == Path::new("/")
    }
}

/// Get the parent directory of a path
pub fn get_parent_path(path: &Path) -> Option<PathBuf> {
    if is_root_path(path) {
        None
    } else {
        path.parent().map(|p| p.to_path_buf())
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Check if a file name should be considered hidden
pub fn is_hidden_file(name: &str) -> bool {
    name.starts_with('.')
}

/// Get the default external editor command
pub fn get_default_editor() -> String {
    #[cfg(windows)]
    {
        std::env::var("EDITOR").unwrap_or_else(|_| "notepad.exe".to_string())
    }
    
    #[cfg(not(windows))]
    {
        std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string())
    }
}

/// Convert a path to display string with proper separators
pub fn path_to_display_string(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.to_string_lossy().replace('/', "\\")
    }
    
    #[cfg(not(windows))]
    {
        path.to_string_lossy().replace('\\', "/")
    }
}

/// Check if the current platform supports file permissions
pub fn supports_file_permissions() -> bool {
    #[cfg(unix)]
    { true }
    
    #[cfg(not(unix))]
    { false }
}

/// Get file permissions as a string (Unix-style)
pub fn get_file_permissions(metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        
        let mut perms = String::with_capacity(10);
        
        // File type
        if metadata.is_dir() {
            perms.push('d');
        } else if metadata.file_type().is_symlink() {
            perms.push('l');
        } else {
            perms.push('-');
        }
        
        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
        
        // Group permissions
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
        
        // Other permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
        
        perms
    }
    
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "r--r--r--".to_string()
        } else {
            "rw-rw-rw-".to_string()
        }
    }
}

/// Format file modification time for display (Norton Commander style)
pub fn format_file_time(system_time: std::time::SystemTime) -> String {
    use chrono::{DateTime, Local};
    
    let datetime: DateTime<Local> = system_time.into();
    datetime.format("%b %d, %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_is_hidden_file() {
        assert!(is_hidden_file(".hidden"));
        assert!(is_hidden_file(".bashrc"));
        assert!(!is_hidden_file("visible.txt"));
        assert!(!is_hidden_file("file.hidden"));
    }

    #[test]
    fn test_normalize_path() {
        let test_path = Path::new("test/path");
        let normalized = normalize_path(test_path);
        
        #[cfg(windows)]
        assert_eq!(normalized.to_string_lossy(), "test\\path");
        
        #[cfg(not(windows))]
        assert_eq!(normalized.to_string_lossy(), "test/path");
    }

    #[test]
    fn test_is_root_path() {
        #[cfg(windows)]
        {
            assert!(is_root_path(Path::new("C:\\")));
            assert!(is_root_path(Path::new("D:\\")));
            assert!(!is_root_path(Path::new("C:\\Users")));
        }
        
        #[cfg(not(windows))]
        {
            assert!(is_root_path(Path::new("/")));
            assert!(!is_root_path(Path::new("/home")));
            assert!(!is_root_path(Path::new("/usr/bin")));
        }
    }

    #[test]
    fn test_get_parent_path() {
        assert_eq!(get_parent_path(Path::new("/home/user/file.txt")), Some(PathBuf::from("/home/user")));
        assert_eq!(get_parent_path(Path::new("/")), None);
        
        #[cfg(windows)]
        {
            assert_eq!(get_parent_path(Path::new("C:\\Users\\file.txt")), Some(PathBuf::from("C:\\Users")));
            assert_eq!(get_parent_path(Path::new("C:\\")), None);
        }
    }

    #[test]
    fn test_format_file_time() {
        use std::time::{SystemTime, UNIX_EPOCH, Duration};
        
        // Test with a known timestamp (2023-12-25 15:30:45 UTC)
        let test_time = UNIX_EPOCH + Duration::from_secs(1703520645);
        let formatted = format_file_time(test_time);
        
        // The format should be "MMM dd, HH:mm" in local time
        // We can't test exact values due to timezone differences, but we can test the format
        assert!(formatted.len() >= 12); // "MMM dd, HH:mm" is typically 12-13 chars
        assert!(formatted.contains(',')); // Should contain comma separator
        assert!(formatted.contains(':')); // Should contain time separator
        
        // Test with SystemTime::now() to ensure it doesn't panic
        let now_formatted = format_file_time(SystemTime::now());
        assert!(now_formatted.len() >= 12);
        assert!(now_formatted.contains(','));
        assert!(now_formatted.contains(':'));
        
        // Test with a very old time (1970-01-01)
        let epoch_formatted = format_file_time(UNIX_EPOCH);
        assert!(epoch_formatted.len() >= 12);
        assert!(epoch_formatted.contains(','));
        assert!(epoch_formatted.contains(':'));
    }
} 
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::io::{Read, Write};
use std::fs::File;
use zip::{ZipArchive, ZipWriter, CompressionMethod};
use tar::Archive as TarArchive;
use chrono::{DateTime, Local, TimeZone};

use crate::error::{GeekCommanderError, Result};
use crate::core::ArchiveEntry;

/// Archive handler trait
pub trait ArchiveHandler {
    fn list_entries(&self, path: &str) -> Result<Vec<ArchiveEntry>>;
    fn extract_file(&self, entry_path: &str, output: &mut dyn Write) -> Result<()>;
    fn extract_to_disk(&self, entry_path: &str, output_path: &Path) -> Result<()>;
}

/// ZIP archive handler
pub struct ZipHandler {
    archive_path: PathBuf,
}

impl ZipHandler {
    pub fn new(archive_path: PathBuf) -> Self {
        ZipHandler { archive_path }
    }
}

impl ArchiveHandler for ZipHandler {
    fn list_entries(&self, virtual_path: &str) -> Result<Vec<ArchiveEntry>> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut entries = Vec::new();
        let prefix = if virtual_path.is_empty() { "" } else { virtual_path };
        
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            let name = entry.name();
            
            // Skip entries that are not in the current virtual directory
            if !name.starts_with(prefix) {
                continue;
            }
            
            let relative_name = &name[prefix.len()..];
            if relative_name.is_empty() {
                continue;
            }
            
            // Check if this is a direct child (not nested deeper)
            let path_parts: Vec<&str> = relative_name.trim_start_matches('/').split('/').collect();
            if path_parts.len() == 1 || (path_parts.len() == 2 && path_parts[1].is_empty()) {
                let entry_name = path_parts[0].to_string();
                let is_dir = name.ends_with('/');
                
                let archive_entry = ArchiveEntry {
                    name: entry_name,
                    path: name.to_string(),
                    is_dir,
                    size: entry.size(),
                    modified: {
                        let dt = entry.last_modified();
                        Local
                            .timestamp_opt(946684800, 0) // Jan 1, 2000 as fallback
                            .single()
                            .unwrap_or_else(|| {
                                Local.timestamp_opt(
                                    946684800 + (dt.year() as i64 - 2000) * 365 * 24 * 3600 +
                                    (dt.month() as i64 - 1) * 30 * 24 * 3600 +
                                    (dt.day() as i64 - 1) * 24 * 3600 +
                                    dt.hour() as i64 * 3600 +
                                    dt.minute() as i64 * 60 +
                                    dt.second() as i64,
                                    0
                                ).single().unwrap_or_else(|| Local.timestamp_opt(946684800, 0).single().unwrap())
                            })
                    },
                };
                entries.push(archive_entry);
            }
        }
        
        // Remove duplicates and sort
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        entries.dedup_by(|a, b| a.name == b.name);
        
        Ok(entries)
    }
    
    fn extract_file(&self, entry_path: &str, output: &mut dyn Write) -> Result<()> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut entry = archive.by_name(entry_path)?;
        std::io::copy(&mut entry, output)?;
        
        Ok(())
    }
    
    fn extract_to_disk(&self, entry_path: &str, output_path: &Path) -> Result<()> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut entry = archive.by_name(entry_path)?;
        
        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut output_file = std::fs::File::create(output_path)?;
        std::io::copy(&mut entry, &mut output_file)?;
        
        Ok(())
    }
}

/// TAR archive handler
pub struct TarHandler {
    archive_path: PathBuf,
}

impl TarHandler {
    pub fn new(archive_path: PathBuf) -> Self {
        TarHandler { archive_path }
    }
}

impl ArchiveHandler for TarHandler {
    fn list_entries(&self, virtual_path: &str) -> Result<Vec<ArchiveEntry>> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = tar::Archive::new(file);
        
        let mut entries = Vec::new();
        let prefix = if virtual_path.is_empty() { "" } else { virtual_path };
        
        for entry_result in archive.entries()? {
            let entry = entry_result?;
            let path = entry.path()?;
            let name = path.to_string_lossy();
            
            // Skip entries that are not in the current virtual directory
            if !name.starts_with(prefix) {
                continue;
            }
            
            let relative_name = &name[prefix.len()..];
            if relative_name.is_empty() {
                continue;
            }
            
            // Check if this is a direct child (not nested deeper)
            let path_parts: Vec<&str> = relative_name.trim_start_matches('/').split('/').collect();
            if path_parts.len() == 1 || (path_parts.len() == 2 && path_parts[1].is_empty()) {
                let entry_name = path_parts[0].to_string();
                let header = entry.header();
                let is_dir = header.entry_type().is_dir();
                
                let archive_entry = ArchiveEntry {
                    name: entry_name,
                    path: name.to_string(),
                    is_dir,
                    size: header.size()?,
                    modified: header.mtime()
                        .map(|mtime| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(mtime))
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                };
                entries.push(archive_entry);
            }
        }
        
        // Remove duplicates and sort
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        entries.dedup_by(|a, b| a.name == b.name);
        
        Ok(entries)
    }
    
    fn extract_file(&self, entry_path: &str, output: &mut dyn Write) -> Result<()> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = tar::Archive::new(file);
        
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;
            if path.to_string_lossy() == entry_path {
                std::io::copy(&mut entry, output)?;
                return Ok(());
            }
        }
        
        Err(GeekCommanderError::archive(format!("Entry '{}' not found in archive", entry_path)))
    }
    
    fn extract_to_disk(&self, entry_path: &str, output_path: &Path) -> Result<()> {
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = tar::Archive::new(file);
        
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;
            if path.to_string_lossy() == entry_path {
                // Create parent directories if needed
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                let mut output_file = std::fs::File::create(output_path)?;
                std::io::copy(&mut entry, &mut output_file)?;
                return Ok(());
            }
        }
        
        Err(GeekCommanderError::archive(format!("Entry '{}' not found in archive", entry_path)))
    }
}

/// Create an appropriate archive handler for the given file
pub fn create_archive_handler(archive_path: &Path) -> Result<Box<dyn ArchiveHandler>> {
    let extension = archive_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "zip" => Ok(Box::new(ZipHandler::new(archive_path.to_path_buf()))),
        "tar" | "tgz" | "gz" => {
            // Check if it's a .tar.gz or .tar.bz2
            let name = archive_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            if name.ends_with(".tar.gz") || name.ends_with(".tgz") || 
               name.ends_with(".tar.bz2") || name.ends_with(".tbz2") ||
               name.ends_with(".tar") {
                Ok(Box::new(TarHandler::new(archive_path.to_path_buf())))
            } else {
                Err(GeekCommanderError::archive(format!("Unsupported archive format: {}", name)))
            }
        }
        _ => Err(GeekCommanderError::archive(format!("Unsupported archive format: {}", extension)))
    }
}

/// Check if a file is a supported archive
pub fn is_supported_archive(path: &Path) -> bool {
    let name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    name.ends_with(".zip") ||
    name.ends_with(".tar") ||
    name.ends_with(".tar.gz") ||
    name.ends_with(".tgz") ||
    name.ends_with(".tar.bz2") ||
    name.ends_with(".tbz2")
}

/// Add files to a ZIP archive
pub fn add_to_zip_archive(archive_path: &Path, files: &[&Path]) -> Result<()> {
    let file = if archive_path.exists() {
        std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(archive_path)?
    } else {
        std::fs::File::create(archive_path)?
    };
    
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    for &file_path in files {
        let name = file_path.file_name()
            .ok_or_else(|| GeekCommanderError::archive("Invalid file name"))?
            .to_string_lossy();
        
        if file_path.is_dir() {
            // Add directory (empty directories need trailing slash)
            zip.add_directory(format!("{}/", name), options)?;
            
            // Recursively add directory contents
            for entry in walkdir::WalkDir::new(file_path) {
                let entry = entry.map_err(|e| GeekCommanderError::archive(format!("Walk error: {}", e)))?;
                let path = entry.path();
                
                if path == file_path {
                    continue; // Skip the root directory itself
                }
                
                let relative_path = path.strip_prefix(file_path)
                    .map_err(|_| GeekCommanderError::archive("Failed to get relative path"))?;
                let archive_path = Path::new(name.as_ref()).join(relative_path);
                let archive_name = archive_path.to_string_lossy();
                
                if path.is_dir() {
                    zip.add_directory(format!("{}/", archive_name), options)?;
                } else {
                    zip.start_file(archive_name.as_ref(), options)?;
                    let mut file = std::fs::File::open(path)?;
                    std::io::copy(&mut file, &mut zip)?;
                }
            }
        } else {
            // Add single file
            zip.start_file(name.as_ref(), options)?;
            let mut file = std::fs::File::open(file_path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }
    
    zip.finish()?;
    Ok(())
} 
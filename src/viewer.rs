use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use crate::error::{GeekCommanderError, Result};
use crate::platform;

const MAX_FILE_SIZE_FOR_VIEWING: u64 = 50 * 1024 * 1024; // 50MB
const BUFFER_SIZE: usize = 64 * 1024; // 64KB

#[derive(Debug, Clone)]
pub struct FileViewer {
    pub content: String,
    pub lines: Vec<String>,
    pub current_line: usize,
    pub scroll_offset: usize,
    pub file_path: String,
    pub file_size: u64,
    pub is_binary: bool,
}

impl FileViewer {
    pub fn new(file_path: &Path) -> Result<Self> {
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        if file_size > MAX_FILE_SIZE_FOR_VIEWING {
            return Err(GeekCommanderError::FileOperation(format!(
                "File is too large to view ({} bytes). Maximum size is {} bytes.",
                file_size, MAX_FILE_SIZE_FOR_VIEWING
            )));
        }

        let mut file = fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check if file is binary
        let is_binary = is_binary_content(&buffer);
        
        if is_binary {
            return Ok(FileViewer {
                content: format!("Binary file - {} bytes\nCannot display binary content.", file_size),
                lines: vec![
                    format!("Binary file - {} bytes", file_size),
                    "Cannot display binary content.".to_string(),
                ],
                current_line: 0,
                scroll_offset: 0,
                file_path: file_path.to_string_lossy().to_string(),
                file_size,
                is_binary: true,
            });
        }

        // Convert to UTF-8, replacing invalid sequences
        let content = String::from_utf8_lossy(&buffer).to_string();
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Ok(FileViewer {
            content,
            lines,
            current_line: 0,
            scroll_offset: 0,
            file_path: file_path.to_string_lossy().to_string(),
            file_size,
            is_binary: false,
        })
    }

    pub fn scroll_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
            if self.current_line < self.scroll_offset {
                self.scroll_offset = self.current_line;
            }
        }
    }

    pub fn scroll_down(&mut self, visible_lines: usize) {
        if self.current_line < self.lines.len().saturating_sub(1) {
            self.current_line += 1;
            if self.current_line >= self.scroll_offset + visible_lines {
                self.scroll_offset = self.current_line - visible_lines + 1;
            }
        }
    }

    pub fn page_up(&mut self, visible_lines: usize) {
        let page_size = visible_lines.saturating_sub(1).max(1);
        self.current_line = self.current_line.saturating_sub(page_size);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, visible_lines: usize) {
        let page_size = visible_lines.saturating_sub(1).max(1);
        self.current_line = (self.current_line + page_size).min(self.lines.len().saturating_sub(1));
        self.scroll_offset = self.scroll_offset + page_size;
        
        // Ensure we don't scroll past the end
        let max_scroll = self.lines.len().saturating_sub(visible_lines);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    pub fn home(&mut self) {
        self.current_line = 0;
        self.scroll_offset = 0;
    }

    pub fn end(&mut self, visible_lines: usize) {
        self.current_line = self.lines.len().saturating_sub(1);
        self.scroll_offset = self.lines.len().saturating_sub(visible_lines);
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Content
                Constraint::Length(1), // Status
            ])
            .split(area);

        // Title
        let title = format!("Viewer: {} ({} bytes)", self.file_path, self.file_size);
        let title_paragraph = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(title_paragraph, chunks[0]);

        // Content
        let visible_lines = chunks[1].height as usize;
        let end_line = (self.scroll_offset + visible_lines).min(self.lines.len());
        
        let visible_content = if self.lines.is_empty() {
            String::new()
        } else {
            self.lines[self.scroll_offset..end_line].join("\n")
        };

        let content_paragraph = Paragraph::new(visible_content)
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));
        f.render_widget(content_paragraph, chunks[1]);

        // Status
        let status = if self.is_binary {
            "Binary file - F10/Esc to exit".to_string()
        } else {
            format!(
                "Line {}/{} | ↑↓ Scroll | PgUp/PgDn Page | Home/End | F10/Esc Exit",
                self.current_line + 1,
                self.lines.len()
            )
        };
        
        let status_paragraph = Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Left);
        f.render_widget(status_paragraph, chunks[2]);
    }

    pub fn handle_key(&mut self, key: KeyCode, visible_lines: usize) -> bool {
        match key {
            KeyCode::F(10) | KeyCode::Esc => return false, // Exit viewer
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(visible_lines),
            KeyCode::PageUp => self.page_up(visible_lines),
            KeyCode::PageDown => self.page_down(visible_lines),
            KeyCode::Home => self.home(),
            KeyCode::End => self.end(visible_lines),
            _ => {} // Ignore other keys
        }
        true // Continue viewing
    }
}

pub fn launch_external_editor(file_path: &Path) -> Result<()> {
    let editor = platform::get_default_editor();
    
    // Exit terminal raw mode before launching editor
    crossterm::terminal::disable_raw_mode()?;
    
    let status = Command::new(&editor)
        .arg(file_path)
        .status();
    
    // Re-enter raw mode after editor exits
    crossterm::terminal::enable_raw_mode()?;
    
    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                Err(GeekCommanderError::FileOperation(format!(
                    "Editor '{}' exited with error", editor
                )))
            }
        }
        Err(e) => Err(GeekCommanderError::FileOperation(format!(
            "Failed to launch editor '{}': {}", editor, e
        ))),
    }
}

fn is_binary_content(buffer: &[u8]) -> bool {
    // Simple binary detection: check for null bytes and high ratio of non-printable characters
    let null_count = buffer.iter().filter(|&&b| b == 0).count();
    if null_count > 0 {
        return true;
    }
    
    let printable_count = buffer.iter()
        .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
        .count();
    
    if buffer.is_empty() {
        return false;
    }
    
    let printable_ratio = printable_count as f64 / buffer.len() as f64;
    printable_ratio < 0.7 // If less than 70% printable, consider binary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content(b"Hello, world!"));
        assert!(!is_binary_content(b"Line 1\nLine 2\nLine 3"));
        assert!(is_binary_content(b"Hello\x00world"));
        assert!(is_binary_content(&[0u8, 1u8, 2u8, 3u8, 4u8]));
        assert!(!is_binary_content(b""));
    }

    #[test]
    fn test_file_viewer_creation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        fs::write(&test_file, content).unwrap();
        
        let viewer = FileViewer::new(&test_file)?;
        
        assert_eq!(viewer.lines.len(), 5);
        assert_eq!(viewer.lines[0], "Line 1");
        assert_eq!(viewer.lines[4], "Line 5");
        assert_eq!(viewer.current_line, 0);
        assert_eq!(viewer.scroll_offset, 0);
        assert!(!viewer.is_binary);
        
        Ok(())
    }

    #[test]
    fn test_viewer_navigation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let content = (0..20).map(|i| format!("Line {}", i + 1)).collect::<Vec<_>>().join("\n");
        fs::write(&test_file, content).unwrap();
        
        let mut viewer = FileViewer::new(&test_file)?;
        let visible_lines = 10;
        
        // Test scroll down
        viewer.scroll_down(visible_lines);
        assert_eq!(viewer.current_line, 1);
        
        // Test page down
        viewer.page_down(visible_lines);
        assert!(viewer.current_line > 1);
        
        // Test home
        viewer.home();
        assert_eq!(viewer.current_line, 0);
        assert_eq!(viewer.scroll_offset, 0);
        
        // Test end
        viewer.end(visible_lines);
        assert_eq!(viewer.current_line, 19); // 20 lines, 0-indexed
        
        Ok(())
    }

    #[test]
    fn test_binary_file_detection() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let binary_file = temp_dir.path().join("binary.dat");
        
        let mut file = File::create(&binary_file).unwrap();
        file.write_all(&[0u8, 1u8, 2u8, 255u8, 128u8]).unwrap();
        
        let viewer = FileViewer::new(&binary_file)?;
        assert!(viewer.is_binary);
        assert!(viewer.content.contains("Binary file"));
        
        Ok(())
    }

    #[test]
    fn test_large_file_rejection() {
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");
        
        // Create a file that's larger than the viewing limit
        let content = "x".repeat((MAX_FILE_SIZE_FOR_VIEWING + 1) as usize);
        fs::write(&large_file, content).unwrap();
        
        let result = FileViewer::new(&large_file);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("too large"));
        }
    }

    #[test]
    fn test_empty_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.txt");
        
        fs::write(&empty_file, "").unwrap();
        
        let viewer = FileViewer::new(&empty_file)?;
        assert_eq!(viewer.lines.len(), 0);
        assert!(!viewer.is_binary);
        
        Ok(())
    }
} 
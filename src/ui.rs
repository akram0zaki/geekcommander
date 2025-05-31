use std::io::Stdout;
use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Table, Row, Cell},
    Frame, Terminal,
};
use crate::config::Config;
use crate::core::{PaneState, FileOperation, copy_files, move_files, delete_files, execute_operation, create_directory, rename_file, FileEntry};
use crate::error::{GeekCommanderError, Result};
use crate::viewer::{FileViewer, launch_external_editor};
use crate::platform;

#[derive(Clone, Debug, PartialEq)]
pub enum DialogType {
    Help,
    Confirm { message: String, action: ConfirmAction },
    Input { prompt: String, input: String, action: InputAction },
    Progress { operation: FileOperation },
    Error { message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction {
    Copy,
    Move,
    Delete,
    Overwrite,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputAction {
    NewDirectory,
    Rename,
    SelectByPattern,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppMode {
    Normal,
    Viewer,
}

pub struct App {
    pub config: Config,
    pub left_pane: PaneState,
    pub right_pane: PaneState,
    pub active_pane: usize,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub current_dialog: Option<DialogType>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub viewer: Option<FileViewer>,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let left_start = config.panels.left.clone();
        let right_start = config.panels.right.clone();

        let left_pane = PaneState::new(left_start)?;
        let right_pane = PaneState::new(right_start)?;

        Ok(App {
            config,
            left_pane,
            right_pane,
            active_pane: 0,
            terminal,
            current_dialog: None,
            should_quit: false,
            mode: AppMode::Normal,
            viewer: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.draw()?;
            
            if self.should_quit {
                break;
            }
            
            // Check for events with a small timeout
            if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
                if let Ok(event) = event::read() {
                    if let event::Event::Key(key) = event {
                        // Only process key press events, ignore key repeat and release
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key.code, key.modifiers)?;
                            // Small delay to prevent key repeat flooding
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
            }
        }
        
        self.cleanup()
    }

    fn draw(&mut self) -> Result<()> {
        let config = self.config.clone();
        let left_pane = self.left_pane.clone();
        let right_pane = self.right_pane.clone();
        let active_pane = self.active_pane;
        let current_dialog = self.current_dialog.clone();
        let mode = self.mode.clone();
        let viewer = self.viewer.clone();
        
        self.terminal.draw(|f| {
            match mode {
                AppMode::Normal => {
                    // Set the main background to blue (Norton Commander style)
                    let main_block = Block::default()
                        .style(Style::default().bg(Color::Blue));
                    f.render_widget(main_block, f.size());

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1), // Title bar
                            Constraint::Min(1),    // Main content
                            Constraint::Length(1), // Status bar
                        ])
                        .split(f.size());

                    // Title bar with blue background and cyan text
                    let title = Paragraph::new("Geek Commander")
                        .style(Style::default().fg(Color::Cyan).bg(Color::Blue))
                        .alignment(Alignment::Center);
                    f.render_widget(title, chunks[0]);

                    // Main content area (dual panes)
                    let main_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(chunks[1]);

                    // Left pane
                    render_pane(f, main_chunks[0], &left_pane, active_pane == 0, &config);
                    
                    // Right pane  
                    render_pane(f, main_chunks[1], &right_pane, active_pane == 1, &config);

                    // Status bar with blue background and cyan text
                    let left_path = platform::path_to_display_string(&left_pane.current_path);
                    let right_path = platform::path_to_display_string(&right_pane.current_path);
                    let free_space = match platform::get_free_disk_space(&left_pane.current_path) {
                        Ok(space) => platform::format_file_size(space),
                        Err(_) => "Unknown".to_string(),
                    };
                    
                    let status_text = format!(
                        "Left: {} | Right: {} | Free: {} | F1 Help | F5 Copy | F6 Move | F7 NewDir | F8 Delete | F10 Exit",
                        left_path, right_path, free_space
                    );
                    
                    let status = Paragraph::new(status_text)
                        .style(Style::default().bg(Color::Blue).fg(Color::Cyan))
                        .alignment(Alignment::Left);
                    f.render_widget(status, chunks[2]);

                    // Render dialog if present
                    if let Some(ref dialog) = current_dialog {
                        render_dialog(f, dialog, &config);
                    }
                },
                AppMode::Viewer => {
                    if let Some(ref viewer) = viewer {
                        viewer.render(f, f.size());
                    }
                },
            }
        })?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.mode {
            AppMode::Viewer => {
                if let Some(ref mut viewer) = self.viewer {
                    let visible_lines = self.terminal.size()?.height as usize - 3; // Account for title and status
                    if !viewer.handle_key(key, visible_lines) {
                        self.mode = AppMode::Normal;
                        self.viewer = None;
                    }
                }
                return Ok(());
            },
            AppMode::Normal => {
                // Handle dialogs first
                if let Some(ref dialog) = self.current_dialog.clone() {
                    return self.handle_dialog_key(key, modifiers, dialog.clone());
                }

                // Calculate pane height for scrolling
                let terminal_size = self.terminal.size()?;
                let pane_height = terminal_size.height.saturating_sub(3) as usize; // Title + status bar

                // Handle core navigation keys directly first (before keybindings)
                match key {
                    KeyCode::Tab => {
                        self.active_pane = if self.active_pane == 0 { 1 } else { 0 };
                        return Ok(());
                    },
                    KeyCode::Up => {
                        self.get_active_pane_mut().cursor_up(pane_height);
                        return Ok(());
                    },
                    KeyCode::Down => {
                        self.get_active_pane_mut().cursor_down(pane_height);
                        return Ok(());
                    },
                    KeyCode::Enter => {
                        self.handle_enter()?;
                        return Ok(());
                    },
                    KeyCode::Backspace => {
                        self.handle_parent_directory()?;
                        return Ok(());
                    },
                    _ => {}
                }

                // Check for configured keybindings
                if self.config.keybindings.help.matches(key, modifiers) {
                    self.current_dialog = Some(DialogType::Help);
                } else if self.config.keybindings.quit.matches(key, modifiers) {
                    self.should_quit = true;
                } else if self.config.keybindings.copy.matches(key, modifiers) {
                    self.handle_copy()?;
                } else if self.config.keybindings.move_files.matches(key, modifiers) {
                    self.handle_move()?;
                } else if self.config.keybindings.delete.matches(key, modifiers) {
                    self.handle_delete()?;
                } else if self.config.keybindings.rename.matches(key, modifiers) {
                    self.handle_rename()?;
                } else if self.config.keybindings.new_dir.matches(key, modifiers) {
                    self.handle_new_directory()?;
                } else if self.config.keybindings.view.matches(key, modifiers) {
                    self.handle_view()?;
                } else if self.config.keybindings.edit.matches(key, modifiers) {
                    self.handle_edit()?;
                } else if self.config.keybindings.select.matches(key, modifiers) {
                    self.handle_select()?;
                } else if self.config.keybindings.select_all.matches(key, modifiers) {
                    self.handle_select_all()?;
                } else if self.config.keybindings.wildcard.matches(key, modifiers) {
                    self.handle_wildcard_select()?;
                } else if self.config.keybindings.reload.matches(key, modifiers) {
                    self.handle_reload_config()?;
                } else {
                    // Handle remaining navigation keys
                    match key {
                        KeyCode::PageUp => {
                            self.get_active_pane_mut().page_up(pane_height);
                        },
                        KeyCode::PageDown => {
                            self.get_active_pane_mut().page_down(pane_height);
                        },
                        KeyCode::Home => {
                            self.get_active_pane_mut().cursor_home(pane_height);
                        },
                        KeyCode::End => {
                            self.get_active_pane_mut().cursor_end(pane_height);
                        },
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_dialog_key(&mut self, key: KeyCode, _modifiers: KeyModifiers, dialog: DialogType) -> Result<()> {
        match dialog {
            DialogType::Help => {
                // Any key closes help
                self.current_dialog = None;
            },
            DialogType::Error { .. } => {
                // Any key closes error dialog
                self.current_dialog = None;
            },
            DialogType::Confirm { action, .. } => {
                match key {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.current_dialog = None;
                        self.execute_confirm_action(action)?;
                    },
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.current_dialog = None;
                    },
                    _ => {}
                }
            },
            DialogType::Input { prompt, mut input, action } => {
                match key {
                    KeyCode::Enter => {
                        self.current_dialog = None;
                        self.execute_input_action(action, &input)?;
                    },
                    KeyCode::Esc => {
                        self.current_dialog = None;
                    },
                    KeyCode::Backspace => {
                        input.pop();
                        self.current_dialog = Some(DialogType::Input { prompt, input, action });
                    },
                    KeyCode::Char(c) => {
                        input.push(c);
                        self.current_dialog = Some(DialogType::Input { prompt, input, action });
                    },
                    _ => {}
                }
            },
            DialogType::Progress { .. } => {
                // Progress dialogs are typically non-interactive
                // Could add cancel functionality here
                if key == KeyCode::Esc {
                    self.current_dialog = None;
                }
            },
        }
        Ok(())
    }

    fn get_active_pane_mut(&mut self) -> &mut PaneState {
        if self.active_pane == 0 {
            &mut self.left_pane
        } else {
            &mut self.right_pane
        }
    }

    fn get_inactive_pane(&self) -> &PaneState {
        if self.active_pane == 0 {
            &self.right_pane
        } else {
            &self.left_pane
        }
    }

    fn handle_enter(&mut self) -> Result<()> {
        let pane = self.get_active_pane_mut();
        if let Some(entry) = pane.entries.get(pane.cursor_index).cloned() {
            if entry.is_dir {
                let new_path = if entry.name == ".." {
                    pane.current_path.parent().unwrap_or(&pane.current_path).to_path_buf()
                } else {
                    pane.current_path.join(&entry.name)
                };
                pane.enter_directory(new_path)?;
            } else if entry.is_archive {
                // TODO: Implement archive navigation
                self.show_error("Archive navigation not yet implemented".to_string());
            } else {
                // Open file in viewer
                self.handle_view()?;
            }
        }
        Ok(())
    }

    fn handle_parent_directory(&mut self) -> Result<()> {
        let pane = self.get_active_pane_mut();
        if let Some(parent) = pane.current_path.parent() {
            if parent != pane.current_path {
                let new_path = parent.to_path_buf();
                pane.enter_directory(new_path)?;
            }
        }
        Ok(())
    }

    fn handle_copy(&mut self) -> Result<()> {
        let current_entry = self.get_active_pane_mut().get_current_entry().cloned();
        let selected = self.get_active_pane_mut().get_selected_entries().len();
        let dest_path = self.get_inactive_pane().current_path.clone();
        
        if selected == 0 {
            if let Some(current) = current_entry {
                if current.name != ".." {
                    let message = format!("Copy '{}' to '{}'?", current.name, dest_path.display());
                    self.current_dialog = Some(DialogType::Confirm {
                        message,
                        action: ConfirmAction::Copy,
                    });
                }
            }
        } else {
            let message = format!("Copy {} selected files to '{}'?", selected, dest_path.display());
            self.current_dialog = Some(DialogType::Confirm {
                message,
                action: ConfirmAction::Copy,
            });
        }
        Ok(())
    }

    fn handle_move(&mut self) -> Result<()> {
        let current_entry = self.get_active_pane_mut().get_current_entry().cloned();
        let selected = self.get_active_pane_mut().get_selected_entries().len();
        let dest_path = self.get_inactive_pane().current_path.clone();
        
        if selected == 0 {
            if let Some(current) = current_entry {
                if current.name != ".." {
                    let message = format!("Move '{}' to '{}'?", current.name, dest_path.display());
                    self.current_dialog = Some(DialogType::Confirm {
                        message,
                        action: ConfirmAction::Move,
                    });
                }
            }
        } else {
            let message = format!("Move {} selected files to '{}'?", selected, dest_path.display());
            self.current_dialog = Some(DialogType::Confirm {
                message,
                action: ConfirmAction::Move,
            });
        }
        Ok(())
    }

    fn handle_delete(&mut self) -> Result<()> {
        let selected = self.get_active_pane_mut().get_selected_entries();
        if selected.is_empty() {
            if let Some(current) = self.get_active_pane_mut().get_current_entry() {
                if current.name != ".." {
                    let message = format!("Delete '{}'?", current.name);
                    self.current_dialog = Some(DialogType::Confirm {
                        message,
                        action: ConfirmAction::Delete,
                    });
                }
            }
        } else {
            let message = format!("Delete {} selected files?", selected.len());
            self.current_dialog = Some(DialogType::Confirm {
                message,
                action: ConfirmAction::Delete,
            });
        }
        Ok(())
    }

    fn handle_rename(&mut self) -> Result<()> {
        if let Some(current) = self.get_active_pane_mut().get_current_entry() {
            if current.name != ".." {
                self.current_dialog = Some(DialogType::Input {
                    prompt: "Rename to:".to_string(),
                    input: current.name.clone(),
                    action: InputAction::Rename,
                });
            }
        }
        Ok(())
    }

    fn handle_new_directory(&mut self) -> Result<()> {
        self.current_dialog = Some(DialogType::Input {
            prompt: "Create directory:".to_string(),
            input: String::new(),
            action: InputAction::NewDirectory,
        });
        Ok(())
    }

    fn handle_view(&mut self) -> Result<()> {
        if let Some(current) = self.get_active_pane_mut().get_current_entry() {
            if !current.is_dir && current.name != ".." {
                match FileViewer::new(&current.path) {
                    Ok(viewer) => {
                        self.viewer = Some(viewer);
                        self.mode = AppMode::Viewer;
                    },
                    Err(e) => {
                        self.show_error(format!("Cannot view file: {}", e));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_edit(&mut self) -> Result<()> {
        if let Some(current) = self.get_active_pane_mut().get_current_entry() {
            if !current.is_dir && current.name != ".." {
                match launch_external_editor(&current.path) {
                    Ok(_) => {
                        // Refresh the pane after editing
                        self.get_active_pane_mut().refresh()?;
                    },
                    Err(e) => {
                        self.show_error(format!("Cannot edit file: {}", e));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_select(&mut self) -> Result<()> {
        self.get_active_pane_mut().toggle_selection();
        Ok(())
    }

    fn handle_select_all(&mut self) -> Result<()> {
        let pane = self.get_active_pane_mut();
        if pane.has_selections() {
            pane.deselect_all();
        } else {
            pane.select_all();
        }
        Ok(())
    }

    fn handle_wildcard_select(&mut self) -> Result<()> {
        self.current_dialog = Some(DialogType::Input {
            prompt: "Select files matching pattern:".to_string(),
            input: "*.".to_string(),
            action: InputAction::SelectByPattern,
        });
        Ok(())
    }

    fn handle_reload_config(&mut self) -> Result<()> {
        match crate::config::Config::load_or_create_default(None) {
            Ok(config) => {
                self.config = config;
                // Could show a success message
            },
            Err(e) => {
                self.show_error(format!("Failed to reload config: {}", e));
            }
        }
        Ok(())
    }

    fn execute_confirm_action(&mut self, action: ConfirmAction) -> Result<()> {
        let dest = self.get_inactive_pane().current_path.clone();
        
        match action {
            ConfirmAction::Copy => {
                let selected = self.get_active_pane_mut().get_selected_entries();
                let sources = if selected.is_empty() {
                    if let Some(current) = self.get_active_pane_mut().get_current_entry() {
                        vec![current]
                    } else {
                        return Ok(());
                    }
                } else {
                    selected
                };
                
                match copy_files(&sources, &dest) {
                    Ok(mut operation) => {
                        // Execute the operation (simplified for now)
                        if let Err(e) = execute_operation(&mut operation) {
                            self.show_error(format!("Copy failed: {}", e));
                        } else {
                            // Refresh both panes
                            self.left_pane.refresh()?;
                            self.right_pane.refresh()?;
                            // Clear selections
                            self.get_active_pane_mut().deselect_all();
                        }
                    },
                    Err(e) => {
                        self.show_error(format!("Copy failed: {}", e));
                    }
                }
            },
            ConfirmAction::Move => {
                let selected = self.get_active_pane_mut().get_selected_entries();
                let sources = if selected.is_empty() {
                    if let Some(current) = self.get_active_pane_mut().get_current_entry() {
                        vec![current]
                    } else {
                        return Ok(());
                    }
                } else {
                    selected
                };
                
                match move_files(&sources, &dest) {
                    Ok(mut operation) => {
                        if let Err(e) = execute_operation(&mut operation) {
                            self.show_error(format!("Move failed: {}", e));
                        } else {
                            self.left_pane.refresh()?;
                            self.right_pane.refresh()?;
                            self.get_active_pane_mut().deselect_all();
                        }
                    },
                    Err(e) => {
                        self.show_error(format!("Move failed: {}", e));
                    }
                }
            },
            ConfirmAction::Delete => {
                let selected = self.get_active_pane_mut().get_selected_entries();
                let sources = if selected.is_empty() {
                    if let Some(current) = self.get_active_pane_mut().get_current_entry() {
                        vec![current]
                    } else {
                        return Ok(());
                    }
                } else {
                    selected
                };
                
                match delete_files(&sources) {
                    Ok(mut operation) => {
                        if let Err(e) = execute_operation(&mut operation) {
                            self.show_error(format!("Delete failed: {}", e));
                        } else {
                            self.get_active_pane_mut().refresh()?;
                            self.get_active_pane_mut().deselect_all();
                        }
                    },
                    Err(e) => {
                        self.show_error(format!("Delete failed: {}", e));
                    }
                }
            },
            ConfirmAction::Overwrite => {
                // Handle file overwrite confirmation
            },
        }
        Ok(())
    }

    fn execute_input_action(&mut self, action: InputAction, input: &str) -> Result<()> {
        match action {
            InputAction::NewDirectory => {
                if !input.trim().is_empty() {
                    let current_path = &self.get_active_pane_mut().current_path;
                    match create_directory(current_path, input.trim()) {
                        Ok(_) => {
                            self.get_active_pane_mut().refresh()?;
                        },
                        Err(e) => {
                            self.show_error(format!("Failed to create directory: {}", e));
                        }
                    }
                }
            },
            InputAction::Rename => {
                if !input.trim().is_empty() {
                    if let Some(current) = self.get_active_pane_mut().get_current_entry() {
                        match rename_file(&current.path, input.trim()) {
                            Ok(_) => {
                                self.get_active_pane_mut().refresh()?;
                            },
                            Err(e) => {
                                self.show_error(format!("Failed to rename: {}", e));
                            }
                        }
                    }
                }
            },
            InputAction::SelectByPattern => {
                if !input.trim().is_empty() {
                    match self.get_active_pane_mut().select_by_pattern(input.trim()) {
                        Ok(count) => {
                            if count == 0 {
                                self.show_error("No files matched the pattern".to_string());
                            }
                        },
                        Err(e) => {
                            self.show_error(format!("Pattern selection failed: {}", e));
                        }
                    }
                }
            },
        }
        Ok(())
    }

    fn show_error(&mut self, message: String) {
        self.current_dialog = Some(DialogType::Error { message });
    }

    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

fn render_pane<B: tui::backend::Backend>(
    f: &mut Frame<B>, 
    area: Rect, 
    pane: &PaneState, 
    is_active: bool, 
    _config: &Config
) {
    // Calculate approximate column widths for right-alignment formatting
    let total_width = area.width.saturating_sub(4); // Account for borders and spacing
    let size_width = (total_width * 15 / 100).max(8) as usize; // 15% of space, minimum 8 chars
    let date_width = (total_width * 20 / 100).max(16) as usize; // 20% of space, minimum 16 chars for "MMM dd, HH:mm"

    // Create rows for the table
    let rows: Vec<Row> = pane.entries.iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut style = if entry.is_dir {
                Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).bg(Color::Blue)
            };

            // Highlight selected items with black background
            if pane.selected_indices.contains(&i) {
                style = style.bg(Color::Black).fg(Color::White);
            }

            let icon = if entry.name == ".." {
                "↑"
            } else if entry.is_dir {
                "📁"
            } else if entry.is_archive {
                "📦"
            } else {
                "📄"
            };

            let name_cell = format!("{} {}", icon, entry.name);
            
            // Right-align size text within its column width
            let size_raw = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                platform::format_file_size(entry.size)
            };
            let size_text = format!("{:>width$}", size_raw, width = size_width);

            // Left-align date text (no padding needed)
            let date_text = platform::format_file_time(entry.modified);

            Row::new(vec![
                Cell::from(name_cell), // Left-aligned name column
                Cell::from(size_text), // Right-aligned size column
                Cell::from(date_text), // Right-aligned date column
            ]).style(style)
        })
        .collect();

    let border_style = if is_active {
        Style::default().fg(Color::Cyan).bg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray).bg(Color::Blue)
    };

    let title = format!("{} ({})", 
        platform::path_to_display_string(&pane.current_path),
        if pane.has_selections() { 
            format!("{} selected", pane.selected_indices.len()) 
        } else { 
            "".to_string() 
        }
    );

    // Create header row with Norton Commander style and right-aligned headers for size/date
    let header_size = format!("{:>width$}", "Size", width = size_width);
    let header_date = "Date"; // Left-aligned header
    
    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from(header_size),
        Cell::from(header_date),
    ])
    .style(Style::default().fg(Color::Yellow).bg(Color::Blue).add_modifier(Modifier::BOLD))
    .bottom_margin(0);

    let table = Table::new(rows)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style)
            .style(Style::default().bg(Color::Blue)))
        .widths(&[
            Constraint::Percentage(65), // Name column gets 65% of space
            Constraint::Percentage(15), // Size column gets 15% of space
            Constraint::Percentage(20), // Date column gets 20% of space
        ])
        .column_spacing(1)
        .style(Style::default().bg(Color::Blue))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD));

    // Create table state with cursor position
    let mut table_state = tui::widgets::TableState::default();
    if !pane.entries.is_empty() && pane.cursor_index < pane.entries.len() {
        table_state.select(Some(pane.cursor_index));
    }

    f.render_stateful_widget(table, area, &mut table_state);
}

fn render_dialog<B: tui::backend::Backend>(f: &mut Frame<B>, dialog: &DialogType, config: &Config) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);
    
    let (title, content) = match dialog {
        DialogType::Help => {
            let help_text = format!(
                "{}     HELP - Geek Commander     {}\n\n\
                F1  - Help                F6  - Move/Rename\n\
                F3  - View File           F7  - New Directory\n\
                F4  - Edit File           F8  - Delete\n\
                F5  - Copy                F10 - Exit\n\n\
                Tab        - Switch Panes\n\
                Enter      - Enter Directory/View File\n\
                Backspace  - Parent Directory\n\
                Insert     - Select/Deselect File\n\
                Ctrl+A     - Select/Deselect All\n\
                *          - Select by Pattern\n\
                Ctrl+R     - Reload Configuration\n\n\
                ↑↓         - Navigate\n\
                PgUp/PgDn  - Page Up/Down\n\
                Home/End   - First/Last Item\n\n\
                Press any key to close",
                "=".repeat(20), "=".repeat(20)
            );
            ("Help", help_text)
        },
        DialogType::Error { message } => ("Error", format!("{}\n\nPress any key to continue", message)),
        DialogType::Confirm { message, .. } => ("Confirm", format!("{}\n\n(Y)es / (N)o", message)),
        DialogType::Input { prompt, input, .. } => ("Input", format!("{}\n{}_", prompt, input)),
        DialogType::Progress { operation } => {
            let progress = if operation.total_size > 0 {
                (operation.processed_size as f64 / operation.total_size as f64 * 100.0) as u16
            } else {
                0
            };
            let current_file = operation.current_file.as_deref().unwrap_or("Unknown");
            let content = format!(
                "Operation: {:?}\nCurrent file: {}\nProgress: {}%\nProcessed: {} / {}",
                operation.operation_type,
                current_file,
                progress,
                platform::format_file_size(operation.processed_size),
                platform::format_file_size(operation.total_size)
            );
            ("Progress", content)
        },
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(config.colors.active_pane_border)))
        .alignment(Alignment::Left)
        .wrap(tui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 
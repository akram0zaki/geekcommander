use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::style::Color;
use crate::error::{GeekCommanderError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub keybindings: Keybindings,
    pub colors: ColorScheme,
    pub panels: PanelConfig,
    pub general: GeneralConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone)]
pub struct Keybindings {
    pub help: KeyBinding,
    pub copy: KeyBinding,
    pub move_files: KeyBinding,
    pub delete: KeyBinding,
    pub rename: KeyBinding,
    pub new_dir: KeyBinding,
    pub quit: KeyBinding,
    pub view: KeyBinding,
    pub edit: KeyBinding,
    pub select: KeyBinding,
    pub select_all: KeyBinding,
    pub wildcard: KeyBinding,
    pub reload: KeyBinding,
    pub switch_pane: KeyBinding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub active_pane_border: Color,
    pub inactive_pane_border: Color,
    pub selected_item: Color,
    pub status_bar: Color,
    pub directory_fg: Color,
    pub file_fg: Color,
    pub cursor_bg: Color,
}

#[derive(Debug, Clone)]
pub struct PanelConfig {
    pub left: PathBuf,
    pub right: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GeneralConfig {
    pub show_hidden: bool,
    pub confirm_delete: bool,
    pub confirm_overwrite: bool,
    pub use_colors: bool,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            keybindings: Keybindings::default(),
            colors: ColorScheme::default(),
            panels: PanelConfig::default(),
            general: GeneralConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Keybindings {
            help: KeyBinding::new(KeyCode::F(1), KeyModifiers::NONE),
            copy: KeyBinding::new(KeyCode::F(5), KeyModifiers::NONE),
            move_files: KeyBinding::new(KeyCode::F(6), KeyModifiers::NONE),
            delete: KeyBinding::new(KeyCode::F(8), KeyModifiers::NONE),
            rename: KeyBinding::new(KeyCode::F(6), KeyModifiers::NONE),
            new_dir: KeyBinding::new(KeyCode::F(7), KeyModifiers::NONE),
            quit: KeyBinding::new(KeyCode::F(10), KeyModifiers::NONE),
            view: KeyBinding::new(KeyCode::F(3), KeyModifiers::NONE),
            edit: KeyBinding::new(KeyCode::F(4), KeyModifiers::NONE),
            select: KeyBinding::new(KeyCode::Insert, KeyModifiers::NONE),
            select_all: KeyBinding::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            wildcard: KeyBinding::new(KeyCode::Char('*'), KeyModifiers::NONE),
            reload: KeyBinding::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            switch_pane: KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            active_pane_border: Color::Cyan,
            inactive_pane_border: Color::Blue,
            selected_item: Color::Black,
            status_bar: Color::Cyan,
            directory_fg: Color::White,
            file_fg: Color::Cyan,
            cursor_bg: Color::Blue,
        }
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        PanelConfig {
            left: home.clone(),
            right: home,
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            show_hidden: false,
            confirm_delete: true,
            confirm_overwrite: true,
            use_colors: true,
            follow_symlinks: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let log_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".geekcommander.log");
        
        LoggingConfig {
            level: "INFO".to_string(),
            file: log_file,
        }
    }
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        KeyBinding { code, modifiers }
    }

    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && self.modifiers == modifiers
    }
}

impl Config {
    pub fn load_or_create_default(config_path: Option<&str>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::get_default_config_path(),
        };

        if config_file.exists() {
            Self::load_from_file(&config_file)
        } else {
            let config = Config::default();
            if let Err(e) = config.save_to_file(&config_file) {
                log::warn!("Failed to create default config file: {}", e);
            }
            Ok(config)
        }
    }

    fn get_default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".geekcommanderrc")
    }

    fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| GeekCommanderError::Config(format!("Failed to read config file: {}", e)))?;

        let mut config = Config::default();
        let sections = parse_ini(&content)?;

        // Parse keybindings
        if let Some(keybindings) = sections.get("Keybindings") {
            config.keybindings = parse_keybindings(keybindings)?;
        }

        // Parse colors
        if let Some(colors) = sections.get("Colors") {
            config.colors = parse_colors(colors)?;
        }

        // Parse panels
        if let Some(panels) = sections.get("Panels") {
            config.panels = parse_panels(panels)?;
        }

        // Parse general settings
        if let Some(general) = sections.get("General") {
            config.general = parse_general(general)?;
        }

        // Parse logging
        if let Some(logging) = sections.get("Logging") {
            config.logging = parse_logging(logging)?;
        }

        Ok(config)
    }

    fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = self.to_ini_string();
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GeekCommanderError::Config(format!("Failed to create config directory: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| GeekCommanderError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    fn to_ini_string(&self) -> String {
        format!(
            "[Keybindings]\n\
            Help=F1\n\
            Copy=F5\n\
            Move=F6\n\
            Delete=F8\n\
            Rename=F6\n\
            NewDir=F7\n\
            Quit=F10\n\
            View=F3\n\
            Edit=F4\n\
            Select=Insert\n\
            SelectAll=Ctrl+A\n\
            Wildcard=*\n\
            Reload=Ctrl+R\n\
            SwitchPane=Tab\n\
            \n\
            [Colors]\n\
            ActivePaneBorder=Yellow\n\
            InactivePaneBorder=Gray\n\
            SelectedItem=Blue\n\
            StatusBar=White\n\
            DirectoryFg=Cyan\n\
            FileFg=White\n\
            CursorBg=DarkGray\n\
            \n\
            [Panels]\n\
            Left={}\n\
            Right={}\n\
            \n\
            [General]\n\
            ShowHidden={}\n\
            ConfirmDelete={}\n\
            ConfirmOverwrite={}\n\
            UseColors={}\n\
            FollowSymlinks={}\n\
            \n\
            [Logging]\n\
            Level={}\n\
            File={}\n",
            self.panels.left.display(),
            self.panels.right.display(),
            self.general.show_hidden,
            self.general.confirm_delete,
            self.general.confirm_overwrite,
            self.general.use_colors,
            self.general.follow_symlinks,
            self.logging.level,
            self.logging.file.display()
        )
    }
}

fn parse_ini(content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut sections = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            sections.insert(current_section.clone(), HashMap::new());
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos+1..].trim().to_string();
            
            if !current_section.is_empty() {
                if let Some(section) = sections.get_mut(&current_section) {
                    section.insert(key, value);
                }
            }
        }
    }

    Ok(sections)
}

fn parse_keybindings(section: &HashMap<String, String>) -> Result<Keybindings> {
    let mut keybindings = Keybindings::default();
    
    for (key, value) in section {
        let binding = parse_key_binding(value)?;
        match key.as_str() {
            "Help" => keybindings.help = binding,
            "Copy" => keybindings.copy = binding,
            "Move" => keybindings.move_files = binding,
            "Delete" => keybindings.delete = binding,
            "Rename" => keybindings.rename = binding,
            "NewDir" => keybindings.new_dir = binding,
            "Quit" => keybindings.quit = binding,
            "View" => keybindings.view = binding,
            "Edit" => keybindings.edit = binding,
            "Select" => keybindings.select = binding,
            "SelectAll" => keybindings.select_all = binding,
            "Wildcard" => keybindings.wildcard = binding,
            "Reload" => keybindings.reload = binding,
            "SwitchPane" => keybindings.switch_pane = binding,
            _ => log::warn!("Unknown keybinding: {}", key),
        }
    }
    
    Ok(keybindings)
}

fn parse_key_binding(value: &str) -> Result<KeyBinding> {
    let parts: Vec<&str> = value.split('+').collect();
    let mut modifiers = KeyModifiers::NONE;
    let mut key_str = value;

    if parts.len() > 1 {
        for part in &parts[..parts.len()-1] {
            match part.to_lowercase().as_str() {
                "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                _ => return Err(GeekCommanderError::Config(format!("Unknown modifier: {}", part))),
            }
        }
        key_str = parts[parts.len()-1];
    }

    let code = match key_str {
        "Tab" => KeyCode::Tab,
        "Enter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Insert" => KeyCode::Insert,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Esc" => KeyCode::Esc,
        s if s.starts_with('F') && s.len() > 1 => {
            let num_str = &s[1..];
            if let Ok(num) = num_str.parse::<u8>() {
                if num >= 1 && num <= 12 {
                    KeyCode::F(num)
                } else {
                    return Err(GeekCommanderError::Config(format!("Invalid function key: {}", s)));
                }
            } else {
                return Err(GeekCommanderError::Config(format!("Invalid function key: {}", s)));
            }
        },
        s if s.len() == 1 => {
            KeyCode::Char(s.chars().next().unwrap())
        },
        _ => return Err(GeekCommanderError::Config(format!("Unknown key: {}", key_str))),
    };

    Ok(KeyBinding::new(code, modifiers))
}

fn parse_colors(section: &HashMap<String, String>) -> Result<ColorScheme> {
    let mut colors = ColorScheme::default();
    
    for (key, value) in section {
        let color = parse_color(value)?;
        match key.as_str() {
            "ActivePaneBorder" => colors.active_pane_border = color,
            "InactivePaneBorder" => colors.inactive_pane_border = color,
            "SelectedItem" => colors.selected_item = color,
            "StatusBar" => colors.status_bar = color,
            "DirectoryFg" => colors.directory_fg = color,
            "FileFg" => colors.file_fg = color,
            "CursorBg" => colors.cursor_bg = color,
            _ => log::warn!("Unknown color setting: {}", key),
        }
    }
    
    Ok(colors)
}

fn parse_color(value: &str) -> Result<Color> {
    match value.to_lowercase().as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "darkgray" | "darkgrey" => Ok(Color::DarkGray),
        "lightred" => Ok(Color::LightRed),
        "lightgreen" => Ok(Color::LightGreen),
        "lightyellow" => Ok(Color::LightYellow),
        "lightblue" => Ok(Color::LightBlue),
        "lightmagenta" => Ok(Color::LightMagenta),
        "lightcyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => Err(GeekCommanderError::Config(format!("Unknown color: {}", value))),
    }
}

fn parse_panels(section: &HashMap<String, String>) -> Result<PanelConfig> {
    let mut panels = PanelConfig::default();
    
    for (key, value) in section {
        match key.as_str() {
            "Left" => panels.left = PathBuf::from(value),
            "Right" => panels.right = PathBuf::from(value),
            _ => log::warn!("Unknown panel setting: {}", key),
        }
    }
    
    Ok(panels)
}

fn parse_general(section: &HashMap<String, String>) -> Result<GeneralConfig> {
    let mut general = GeneralConfig::default();
    
    for (key, value) in section {
        match key.as_str() {
            "ShowHidden" => general.show_hidden = parse_bool(value)?,
            "ConfirmDelete" => general.confirm_delete = parse_bool(value)?,
            "ConfirmOverwrite" => general.confirm_overwrite = parse_bool(value)?,
            "UseColors" => general.use_colors = parse_bool(value)?,
            "FollowSymlinks" => general.follow_symlinks = parse_bool(value)?,
            _ => log::warn!("Unknown general setting: {}", key),
        }
    }
    
    Ok(general)
}

fn parse_logging(section: &HashMap<String, String>) -> Result<LoggingConfig> {
    let mut logging = LoggingConfig::default();
    
    for (key, value) in section {
        match key.as_str() {
            "Level" => logging.level = value.clone(),
            "File" => logging.file = PathBuf::from(value),
            _ => log::warn!("Unknown logging setting: {}", key),
        }
    }
    
    Ok(logging)
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => Err(GeekCommanderError::Config(format!("Invalid boolean value: {}", value))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_binding() {
        let binding = parse_key_binding("F1").unwrap();
        assert_eq!(binding.code, KeyCode::F(1));
        assert_eq!(binding.modifiers, KeyModifiers::NONE);

        let binding = parse_key_binding("Ctrl+A").unwrap();
        assert_eq!(binding.code, KeyCode::Char('A'));
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL);

        let binding = parse_key_binding("Shift+F5").unwrap();
        assert_eq!(binding.code, KeyCode::F(5));
        assert_eq!(binding.modifiers, KeyModifiers::SHIFT);
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("red").unwrap(), Color::Red);
        assert_eq!(parse_color("Blue").unwrap(), Color::Blue);
        assert_eq!(parse_color("YELLOW").unwrap(), Color::Yellow);
        assert!(parse_color("invalid").is_err());
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true").unwrap(), true);
        assert_eq!(parse_bool("false").unwrap(), false);
        assert_eq!(parse_bool("yes").unwrap(), true);
        assert_eq!(parse_bool("no").unwrap(), false);
        assert_eq!(parse_bool("1").unwrap(), true);
        assert_eq!(parse_bool("0").unwrap(), false);
        assert!(parse_bool("invalid").is_err());
    }

    #[test]
    fn test_key_binding_matches() {
        let binding = KeyBinding::new(KeyCode::F(1), KeyModifiers::NONE);
        assert!(binding.matches(KeyCode::F(1), KeyModifiers::NONE));
        assert!(!binding.matches(KeyCode::F(2), KeyModifiers::NONE));
        assert!(!binding.matches(KeyCode::F(1), KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_ini() {
        let content = r#"
            [Section1]
            Key1=Value1
            Key2=Value2

            [Section2]
            Key3=Value3
            ; This is a comment
            Key4=Value4
        "#;

        let sections = parse_ini(content).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections["Section1"]["Key1"], "Value1");
        assert_eq!(sections["Section1"]["Key2"], "Value2");
        assert_eq!(sections["Section2"]["Key3"], "Value3");
        assert_eq!(sections["Section2"]["Key4"], "Value4");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.keybindings.help.code, KeyCode::F(1));
        assert_eq!(config.colors.active_pane_border, Color::Cyan);
        assert_eq!(config.general.show_hidden, false);
        assert_eq!(config.general.confirm_delete, true);
    }
} 
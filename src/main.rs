use log::info;

mod config;
mod error;
mod core;
mod ui;
mod platform;
mod viewer;

use ui::App;
use config::Config;
use error::Result;

/// Main entry point for Geek Commander
fn main() -> Result<()> {
    // Initialize logger
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stderr())
        .apply()
        .map_err(|e| error::GeekCommanderError::Config(format!("Failed to init logger: {}", e)))?;

    info!("Starting Geek Commander");

    // Load configuration
    let config = Config::load_or_create_default(None)?;
    
    // Create and run the application
    let mut app = App::new(config)?;
    app.run()
} 
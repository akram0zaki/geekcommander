use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GeekCommanderError>;

#[derive(Error, Debug)]
pub enum GeekCommanderError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Invalid file type")]
    InvalidFileType,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration value: {0}")]
    InvalidConfig(String),

    #[error("Archive format not supported: {0}")]
    UnsupportedArchiveFormat(String),

    #[error("Cannot extract to this location: {0}")]
    InvalidExtractionPath(String),

    #[error("File operation failed: {0}")]
    FileOperation(String),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("Terminal error: {0}")]
    Terminal(String),
} 
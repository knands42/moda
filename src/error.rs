use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ModManagerError {
    GameNotFound(String),
    PathDiscoveryFailed(String),
    InvalidConfiguration(String),
    PathNotFound(PathBuf),

    IoError(io::Error),
    InvalidMod(String),
    NexusApiError(String),
}

impl std::fmt::Display for ModManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModManagerError::GameNotFound(game) => write!(f, "Game not found: {}", game),
            ModManagerError::PathNotFound(path) => {
                write!(f, "Path not found: {}", path.to_string_lossy())
            }
            ModManagerError::InvalidConfiguration(err) => {
                write!(f, "Invalid configuration: {}", err)
            }
            ModManagerError::PathDiscoveryFailed(path) => {
                write!(f, "Path discovery failed: {}", path)
            }
            ModManagerError::IoError(e) => write!(f, "IO error: {}", e),
            ModManagerError::InvalidMod(s) => write!(f, "Invalid mod: {}", s),
            ModManagerError::NexusApiError(s) => write!(f, "Nexus API error: {}", s),
        }
    }
}

pub type Result<T> = std::result::Result<T, ModManagerError>;

impl From<std::io::Error> for ModManagerError {
    fn from(e: std::io::Error) -> Self {
        ModManagerError::IoError(e)
    }
}

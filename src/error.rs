use std::env::VarError;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ModManagerError {
    GameNotFound(String),
    PathDiscoveryFailed(String),
    InvalidConfiguration(String),
    InvalidFilename(String),
    PathNotFound(PathBuf),
    ModNotFound(String),
    InvalidMod(String),

    IoError(io::Error),
    NexusApiError(String),
    GameSetupFailed(String),
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
            ModManagerError::InvalidFilename(filename) => {
                write!(f, "Invalid filename: {}", filename)
            }
            ModManagerError::ModNotFound(filename) => {
                write!(f, "Mod not found: {}", filename)
            }
            ModManagerError::PathDiscoveryFailed(path) => {
                write!(f, "Path discovery failed: {}", path)
            }
            ModManagerError::IoError(e) => write!(f, "IO error: {}", e),
            ModManagerError::InvalidMod(s) => write!(f, "Invalid mod: {}", s),
            ModManagerError::NexusApiError(s) => write!(f, "Nexus API error: {}", s),
            ModManagerError::GameSetupFailed(s) => write!(f, "Game setup failed: {}", s),
        }
    }
}

pub type Result<T> = std::result::Result<T, ModManagerError>;

impl From<io::Error> for ModManagerError {
    fn from(e: io::Error) -> Self {
        ModManagerError::IoError(e)
    }
}

impl From<VarError> for ModManagerError {
    fn from(e: VarError) -> Self {
        ModManagerError::InvalidConfiguration(e.to_string())
    }
}

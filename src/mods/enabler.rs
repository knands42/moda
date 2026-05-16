use crate::error::ModManagerError;
use std::path::Path;

pub struct Enabler;

impl Enabler {
    pub fn activate(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        if !source.exists() {
            return Err(ModManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", source.display()),
            )));
        }

        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(ModManagerError::IoError)?;
            }
        }

        // Remove existing symlink if present
        if target.is_symlink() {
            std::fs::remove_file(target).map_err(ModManagerError::IoError)?;
        }

        // Error if target exists and is not a symlink
        if target.exists() {
            return Err(ModManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Target path already exists: {}", target.display()),
            )));
        }

        std::os::unix::fs::symlink(source, target).map_err(ModManagerError::IoError)
    }

    pub fn deactivate(mod_path: &Path) -> Result<(), ModManagerError> {
        if !mod_path.exists() {
            return Err(ModManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", mod_path.display()),
            )));
        }

        if mod_path.is_symlink() {
            std::fs::remove_file(mod_path).map_err(ModManagerError::IoError)?;
        }

        Ok(())
    }
}

use crate::error::ModManagerError;
use crate::mods::enabler::Enabler;
use std::path::Path;

pub struct SymlinkEnabler;

impl Enabler for SymlinkEnabler {
    fn activate(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        if !source.exists() {
            log::error!("Activation source not found: {}", source.display());
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
            log::debug!("Removing existing symlink at {}", target.display());
            std::fs::remove_file(target).map_err(ModManagerError::IoError)?;
        }

        // Error if target exists and is not a symlink
        if target.exists() {
            log::error!("Activation target already exists: {}", target.display());
            return Err(ModManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Target path already exists: {}", target.display()),
            )));
        }

        log::debug!(
            "Creating symlink {} -> {}",
            target.display(),
            source.display()
        );
        std::os::unix::fs::symlink(source, target).map_err(ModManagerError::IoError)
    }

    fn deactivate(mod_path: &Path) -> Result<(), ModManagerError> {
        if mod_path.is_symlink() {
            log::debug!("Removing symlink at {}", mod_path.display());
            std::fs::remove_file(mod_path).map_err(ModManagerError::IoError)?;
            return Ok(());
        }

        if !mod_path.exists() {
            log::warn!("Deactivation target not found: {}", mod_path.display());
            return Err(ModManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", mod_path.display()),
            )));
        }

        log::error!(
            "Deactivation target is not a symlink: {}",
            mod_path.display()
        );
        Err(ModManagerError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a symlink: {}", mod_path.display()),
        )))
    }
}

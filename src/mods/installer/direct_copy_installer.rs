use crate::error::ModManagerError;
use crate::mods::installer::Installer;
use std::io;
use std::path::Path;

pub struct DirectCopyInstaller;

impl Installer for DirectCopyInstaller {
    fn get_mod_name_from_installer(path: &Path) -> Result<Option<String>, ModManagerError> {
        Ok(path.file_name().map(|n| n.to_string_lossy().to_string()))
    }

    fn install(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        log::info!(
            "Installing dir {} -> {}",
            source.display(),
            target.display()
        );
        Self::copy_dir_recursive(source, target)
    }

    fn uninstall(file_path: &Path) -> Result<(), crate::error::ModManagerError> {
        match std::fs::remove_dir_all(file_path) {
            Ok(_) => {
                log::info!("Uninstalled {}", file_path.display());
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::warn!("Uninstall target not found: {}", file_path.display());
                Ok(())
            }
            Err(e) => Err(ModManagerError::IoError(e)),
        }
    }
}

impl DirectCopyInstaller {
    fn copy_dir_recursive(folder_src: &Path, folder_dst: &Path) -> Result<(), ModManagerError> {
        std::fs::create_dir_all(folder_dst)?;

        for entry in std::fs::read_dir(folder_src)? {
            let entry = entry?;
            let src_file_path = entry.path();
            let dst_file_path = folder_dst.join(entry.file_name());

            if src_file_path.is_dir() {
                Self::copy_dir_recursive(&src_file_path, &dst_file_path)?;
            } else {
                std::fs::copy(&src_file_path, &dst_file_path)?;
            }
        }

        Ok(())
    }
}

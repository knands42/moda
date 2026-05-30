use crate::error::ModManagerError;
use crate::mods::installer::Installer;
use std::fs::File;
use std::io;
use std::path::Path;
use zip::ZipArchive;

/// Strips the `.zip` extension from a filename, returning the base name.
pub fn strip_zip_ext(name: &str) -> String {
    name.strip_suffix(".zip").unwrap_or(name).to_string()
}

pub struct ZipInstaller;

impl Installer for ZipInstaller {
    /// Checks if a zip wraps its content in a single top-level directory.
    /// Returns `Some(dir_name)` if it wraps, `None` if files are scattered at root.
    fn get_mod_name_from_installer(zip_path: &Path) -> Result<Option<String>, ModManagerError> {
        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        let mut top_names: Vec<String> = Vec::new();
        let mut has_subdir = false;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| {
                ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
            })?;

            let Some(path) = entry.enclosed_name() else {
                continue;
            };
            let components: Vec<_> = path.components().collect();
            let Some(first) = components.first() else {
                continue;
            };
            let name = first.as_os_str().to_string_lossy().to_string();
            if !top_names.contains(&name) {
                top_names.push(name);
            }
            if components.len() > 1 {
                has_subdir = true;
            }
        }

        let result = if top_names.len() == 1 && has_subdir {
            Some(top_names[0].clone())
        } else {
            None
        };

        log::debug!("Zip wrap analysis for {}: {:?}", zip_path.display(), result);
        Ok(result)
    }

    fn install(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        log::info!(
            "Installing zip {} -> {}",
            source.display(),
            target.display()
        );
        Self::install_from_zip(source, target)?;

        log::info!("Install complete: {} entries", count_entries(target));
        Ok(())
    }

    fn uninstall(_file_path: &Path) -> Result<(), ModManagerError> {
        todo!()
    }
}

impl ZipInstaller {
    fn install_from_zip(file_path: &Path, target: &Path) -> Result<(), ModManagerError> {
        let file = File::open(file_path)
            .map_err(|e| ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        let _i = archive.len();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
            })?;

            // Safe extraction: skips paths with ".." or absolute paths
            let Some(name) = entry.enclosed_name() else {
                continue;
            };
            let out_path = target.join(name);

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| {
                    ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                })?
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                    })?;
                }
                let mut out_file = File::create(&out_path).map_err(|e| {
                    ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                })?;
                std::io::copy(&mut entry, &mut out_file).map_err(|e| {
                    ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                })?;
            }
        }

        Ok(())
    }
}

fn count_entries(path: &Path) -> usize {
    if path.is_dir() {
        std::fs::read_dir(path)
            .map(|d| d.flatten().count())
            .unwrap_or(0)
    } else {
        1
    }
}

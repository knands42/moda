use crate::error::ModManagerError;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Strips the `.zip` extension from a filename, returning the base name.
pub fn strip_zip_ext(name: &str) -> String {
    name.strip_suffix(".zip").unwrap_or(name).to_string()
}

pub enum ModSource {
    LocalZip(PathBuf),
    LocalDir(PathBuf),
}

pub struct Installer;

impl Installer {
    /// Checks if a zip wraps its content in a single top-level directory.
    /// Returns `Some(dir_name)` if it wraps, `None` if files are scattered at root.
    pub fn zip_wrap_directory(zip_path: &Path) -> Result<Option<String>, ModManagerError> {
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

        if top_names.len() == 1 && has_subdir {
            Ok(Some(top_names[0].clone()))
        } else {
            Ok(None)
        }
    }

    pub fn install(source: &ModSource, target: &Path) -> Result<(), ModManagerError> {
        match source {
            ModSource::LocalZip(file_path) => Self::install_from_zip(file_path, target),
            ModSource::LocalDir(file_path) => Self::install_from_dir(file_path, target),
        }?;

        Ok(())
    }

    pub fn uninstall_from_dir(file_path: &Path) -> Result<(), ModManagerError> {
        match std::fs::remove_dir_all(file_path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(ModManagerError::IoError(e)),
        }
    }

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

    fn install_from_dir(folder_path: &Path, target: &Path) -> Result<(), ModManagerError> {
        Self::copy_dir_recursive(folder_path, target)
    }

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

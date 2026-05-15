use crate::error::ModManagerError;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub enum ModSource {
    LocalZip(PathBuf),
    LocalDir(PathBuf),
}

pub struct Installer;

impl Installer {
    pub fn install(source: &ModSource, target: &Path) -> Result<(), ModManagerError> {
        match source {
            ModSource::LocalZip(file_path) => Self::install_from_zip(file_path, target),
            ModSource::LocalDir(file_path) => Self::install_from_dir(file_path, target),
        }?;

        Ok(())
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

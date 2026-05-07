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
            ModSource::LocalZip(path) => Self::install_from_zip(path, target),
            ModSource::LocalDir(path) => Self::install_from_dir(path, target),
        }?;

        Ok(())
    }

    fn install_from_zip(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        let file = File::open(source)
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

    fn install_from_dir(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        Self::copy_dir_recursive(source, target)
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ModManagerError> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }
}

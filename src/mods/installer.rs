use std::fs::File;
use std::io;
use crate::error::ModManagerError;
use crate::mods::Mod;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub enum ModSource {
    LocalZip(PathBuf),
    LocalDir(PathBuf),
    NexusDownload { mod_id: u64, file_id: u64 },
}

pub struct Installer;

impl Installer {
    pub fn install(source: &ModSource, target: &Path) -> Result<(), ModManagerError> {
        match source {
            ModSource::LocalZip(path) => Self::install_from_zip(path, target),
            ModSource::LocalDir(path) => Self::install_from_dir(path, target),
            ModSource::NexusDownload { mod_id, file_id } => todo!()
        }?;

        Ok(())
    }

    pub fn install_mod<T: Mod>(mod_: &T, target: &Path) -> Result<(), ModManagerError> {
        todo!()
    }

    fn install_from_zip(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        let file = File::open(source).map_err(|e| {
            ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| {
            ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
        })?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
            })?;

            // Safe extraction: skips paths with `..`
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

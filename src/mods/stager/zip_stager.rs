use crate::error::ModManagerError;
use crate::mods::stager::Stager;
use crate::mods::types::{ModEntry, ModEntryKind};
use std::fs::File;
use std::io;
use std::path::Path;
use zip::ZipArchive;

/// Strips the `.zip` extension from a filename, returning the base name.
pub fn strip_zip_ext(name: &str) -> String {
    name.strip_suffix(".zip").unwrap_or(name).to_string()
}

pub struct ZipStager;

impl Stager for ZipStager {
    fn get_mod_name(zip_path: &Path) -> Result<String, ModManagerError> {
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
            let result = top_names[0].clone();
            log::debug!("Zip wrap analysis for {}: {:?}", zip_path.display(), result);
            Ok(result)
        } else {
            Err(ModManagerError::InvalidMod(format!(
                "Zip {} does not have a single wrapping directory",
                zip_path.display()
            )))
        }
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

    fn stage(entry: &ModEntry, staging_path: &Path) -> Result<ModEntry, ModManagerError> {
        let (name, target) = match Self::get_mod_name(&entry.path) {
            Ok(dir) => (dir, staging_path.to_path_buf()),
            Err(_) => {
                let name = strip_zip_ext(&entry.name);
                (name.clone(), staging_path.join(&name))
            }
        };
        Self::install(&entry.path, &target)?;
        Ok(ModEntry {
            name: name.clone(),
            path: staging_path.join(&name),
            kind: ModEntryKind::Directory,
            metadata: None,
        })
    }
}

impl ZipStager {
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

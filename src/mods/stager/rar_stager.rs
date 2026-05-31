use crate::error::ModManagerError;
use crate::mods::stager::Stager;
use crate::mods::types::{ModEntry, ModEntryKind};
use std::io;
use std::path::Path;
use unrar::Archive;

pub fn strip_rar_ext(name: &str) -> String {
    name.strip_suffix(".rar").unwrap_or(name).to_string()
}

pub struct RarStager;

impl Stager for RarStager {
    fn get_mod_name(rar_path: &Path) -> Result<String, ModManagerError> {
        let archive = Archive::new(rar_path)
            .open_for_listing()
            .map_err(|e| ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        let mut top_names: Vec<String> = Vec::new();
        let mut has_subdir = false;

        for result in archive {
            let entry = result.map_err(|e| {
                ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
            })?;

            let components: Vec<_> = entry.filename.components().collect();
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
            Ok(top_names[0].clone())
        } else {
            Err(ModManagerError::InvalidMod(format!(
                "RAR {} does not have a single wrapping directory",
                rar_path.display()
            )))
        }
    }

    fn stage(entry: &ModEntry, staging_path: &Path) -> Result<ModEntry, ModManagerError> {
        let (name, target) = match Self::get_mod_name(&entry.path) {
            Ok(dir) => (dir, staging_path.to_path_buf()),
            Err(_) => {
                let name = strip_rar_ext(&entry.name);
                (name.clone(), staging_path.join(&name))
            }
        };
        Self::install_from_rar(&entry.path, &target)?;
        Ok(ModEntry {
            name: name.clone(),
            path: staging_path.join(&name),
            kind: ModEntryKind::Directory,
            metadata: None,
        })
    }
}

impl RarStager {
    fn install_from_rar(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        log::info!(
            "Installing rar {} -> {}",
            source.display(),
            target.display()
        );

        let archive = Archive::new(source)
            .open_for_processing()
            .map_err(|e| ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        Self::process_archive(archive, target)
    }

    fn process_archive(
        mut archive: unrar::OpenArchive<unrar::Process, unrar::CursorBeforeHeader>,
        target: &Path,
    ) -> Result<(), ModManagerError> {
        loop {
            let header_archive = match archive.read_header() {
                Ok(Some(a)) => a,
                Ok(None) => break,
                Err(e) => {
                    return Err(ModManagerError::IoError(io::Error::new(
                        io::ErrorKind::InvalidData,
                        e,
                    )))
                }
            };

            let out_path = target.join(&header_archive.entry().filename);

            if header_archive.entry().is_directory() {
                std::fs::create_dir_all(&out_path)?;
                archive = header_archive.skip().map_err(|e| {
                    ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                })?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let (data, next_archive) = header_archive.read().map_err(|e| {
                    ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
                })?;
                std::fs::write(&out_path, &data)?;
                archive = next_archive;
            }
        }

        Ok(())
    }
}

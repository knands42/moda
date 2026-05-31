use crate::error::ModManagerError;
use crate::mods::stager::Stager;
use crate::mods::{strip_rar_ext, ModEntry, ModEntryKind};
use std::io;
use std::path::Path;

pub struct DirectCopyStager;

impl Stager for DirectCopyStager {
    fn get_mod_name(path: &Path) -> Result<String, ModManagerError> {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| {
                ModManagerError::InvalidMod(format!("Path has no file name: {}", path.display()))
            })
    }

    fn stage(entry: &ModEntry, staging_path: &Path) -> Result<ModEntry, ModManagerError> {
        let target = staging_path.join(&entry.name);
        Self::copy_dir_recursive(&entry.path, &target)?;
        Ok(ModEntry {
            name: entry.name.clone(),
            path: target,
            kind: ModEntryKind::Directory,
            metadata: None,
        })
    }
}

impl DirectCopyStager {
    fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        log::info!(
            "Installing dir {} -> {}",
            source.display(),
            target.display()
        );

        std::fs::create_dir_all(target)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let src_file_path = entry.path();
            let dst_file_path = target.join(entry.file_name());

            if src_file_path.is_dir() {
                Self::copy_dir_recursive(&src_file_path, &dst_file_path)?;
            } else {
                std::fs::copy(&src_file_path, &dst_file_path)?;
            }
        }

        Ok(())
    }
}

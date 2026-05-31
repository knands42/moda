pub mod direct_copy_stager;
pub mod rar_stager;
pub mod zip_stager;

use crate::error::ModManagerError;
use crate::mods::types::{ModEntry, ModEntryKind};
pub use direct_copy_stager::*;
pub use rar_stager::{strip_rar_ext, RarStager};
use std::path::Path;
pub use zip_stager::{strip_zip_ext, ZipStager};

pub trait Stager {
    fn get_mod_name(path: &Path) -> Result<String, ModManagerError>;
    fn install(source: &Path, target: &Path) -> Result<(), ModManagerError>;
    fn unstage(file_path: &Path) -> Result<(), ModManagerError>;

    fn stage(entry: &ModEntry, staging_path: &Path) -> Result<ModEntry, ModManagerError> {
        let target = staging_path.join(&entry.name);
        Self::install(entry.path.as_path(), &target)?;
        Ok(ModEntry {
            name: entry.name.clone(),
            path: target,
            kind: ModEntryKind::Directory,
            metadata: None,
        })
    }
}

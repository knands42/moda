pub mod direct_copy_stager;
pub mod rar_stager;
pub mod zip_stager;

use crate::error::ModManagerError;
use crate::mods::types::{ModEntry, ModEntryKind};
pub use direct_copy_stager::*;
pub use rar_stager::{strip_rar_ext, RarStager};
use std::io;
use std::path::Path;
pub use zip_stager::{strip_zip_ext, ZipStager};

pub trait Stager {
    fn get_mod_name(path: &Path) -> Result<String, ModManagerError>;
    fn unstage(file_path: &Path) -> Result<(), ModManagerError> {
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
    fn stage(entry: &ModEntry, staging_path: &Path) -> Result<ModEntry, ModManagerError>;
}

pub mod catalog;
pub mod downloader;
pub mod enabler;
pub mod installer;
mod mod_state;
mod orchestrator;

use crate::error::ModManagerError;
use crate::mods::catalog::ModEntry;
use std::path::Path;

pub use enabler::SymlinkEnabler;
pub use installer::{strip_zip_ext, DirectCopyInstaller, Installer, ZipInstaller};
pub use mod_state::ModState;
pub use orchestrator::{SyncManager, SyncManagerOps};

#[derive(Debug, Clone, PartialEq)]
pub enum ModEntryKind {
    Directory,
    ZipArchive,
    Other,
}

impl ModEntryKind {
    pub(crate) fn stage(
        &self,
        mod_entry: &ModEntry,
        staging_path: &Path,
    ) -> Result<ModEntry, ModManagerError> {
        todo!()
    }
}

pub fn allowed_extensions() -> &'static [&'static str] {
    static ALLOWED_EXTENSIONS: &[&str] = &["zip", "rar"];
    ALLOWED_EXTENSIONS
}

pub fn map_ext_to_kind(ext: &str) -> ModEntryKind {
    match ext {
        "zip" => ModEntryKind::ZipArchive,
        "rar" => ModEntryKind::ZipArchive,
        _ => ModEntryKind::Other,
    }
}
//
// pub fn map_ext_to_installer(kind: ModEntryKind) -> Enabler {
//     match kind {
//     }
// }

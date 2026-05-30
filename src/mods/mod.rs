pub mod catalog;
pub mod downloader;
pub mod enabler;
mod mod_state;
mod orchestrator;
pub mod stager;
pub mod types;

use std::path::Path;

use crate::error::ModManagerError;

pub use enabler::SymlinkEnabler;
pub use mod_state::ModState;
pub use orchestrator::{SyncManager, SyncManagerOps};
pub use stager::{strip_zip_ext, DirectCopyStager, Stager, ZipStager};
pub use types::{allowed_extensions, map_ext_to_kind, ModEntry, ModEntryKind, ModMetadata};

impl ModEntryKind {
    pub(crate) fn stage(
        self,
        mod_entry: &ModEntry,
        staging_path: &Path,
    ) -> Result<Option<ModEntry>, ModManagerError> {
        match self {
            ModEntryKind::Directory => {
                let entry = DirectCopyStager::stage(mod_entry, staging_path)?;
                Ok(Some(entry))
            }
            ModEntryKind::ZipArchive => {
                let entry = ZipStager::stage(mod_entry, staging_path)?;
                Ok(Some(entry))
            }
            ModEntryKind::Other => Ok(None),
        }
    }
}

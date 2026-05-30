pub mod direct_copy_enabler;
mod pak_enabler;
pub mod symlink_enabler;

use crate::error::ModManagerError;
use crate::mods::types::{ModEntry, ModEntryKind};
pub use direct_copy_enabler::DirectCopyEnabler;
use std::path::Path;
pub use symlink_enabler::SymlinkEnabler;

pub trait Enabler: Send + Sync {
    fn activate(source: &Path, target: &Path) -> Result<(), ModManagerError>;
    fn deactivate(target: &Path) -> Result<(), ModManagerError>;

    fn enable(entry: &ModEntry, target_dir: &Path) -> Result<ModEntry, ModManagerError> {
        let target = target_dir.join(&entry.name);
        Self::activate(entry.path.as_path(), target.as_path())?;
        Ok(ModEntry {
            name: entry.name.clone(),
            path: target,
            kind: ModEntryKind::Directory,
            metadata: None,
        })
    }

    fn disable(entry: &ModEntry) -> Result<(), ModManagerError> {
        Self::deactivate(&entry.path)
    }
}

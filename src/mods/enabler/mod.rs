pub mod direct_copy_enabler;
mod pak_enabler;
pub mod symlink_enabler;

use crate::error::ModManagerError;
pub use direct_copy_enabler::DirectCopyEnabler;
pub use symlink_enabler::SymlinkEnabler;

pub trait Enabler: Send + Sync {
    fn activate(source: &std::path::Path, target: &std::path::Path) -> Result<(), ModManagerError>;
    fn deactivate(target: &std::path::Path) -> Result<(), ModManagerError>;
}

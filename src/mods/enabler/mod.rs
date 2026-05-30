pub mod symlink_enabler;
mod pak_enabler;
pub mod direct_copy_enabler;

pub use symlink_enabler::SymlinkEnabler;
pub use direct_copy_enabler::DirectCopyEnabler;
use crate::error::ModManagerError;

pub trait Enabler: Send + Sync {
    fn activate(source: &std::path::Path, target: &std::path::Path) -> Result<(), ModManagerError>;
    fn deactivate(target: &std::path::Path) -> Result<(), ModManagerError>;
}
mod symlink_enabler;
mod pak_enabler;
mod direct_copy_enabler;

pub use symlink_enabler::SymlinkEnabler;
use crate::error::ModManagerError;

pub trait Enabler: Send + Sync {
    fn activate(source: &std::path::Path, target: &std::path::Path) -> Result<(), ModManagerError>;
    fn deactivate(target: &std::path::Path) -> Result<(), ModManagerError>;
}
pub mod direct_copy_installer;
pub mod zip_installer;

use crate::error::ModManagerError;
pub use direct_copy_installer::*;
use std::path::Path;
pub use zip_installer::*;

pub trait Installer {
    // TODO: change result type to avoid option
    fn get_mod_name_from_installer(path: &Path) -> Result<Option<String>, ModManagerError>;
    fn install(source: &Path, target: &Path) -> Result<(), ModManagerError>;
    fn uninstall(file_path: &Path) -> Result<(), ModManagerError>;
}

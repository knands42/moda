use crate::error::ModManagerError;
use crate::mods::enabler::Enabler;
use std::path::Path;

pub struct DirectCopyEnabler;

impl Enabler for DirectCopyEnabler {
    fn activate(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        todo!()
    }

    fn deactivate(target: &Path) -> Result<(), ModManagerError> {
        todo!()
    }
}

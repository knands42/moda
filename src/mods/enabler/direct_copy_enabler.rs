use std::path::Path;
use crate::error::ModManagerError;
use crate::mods::enabler::Enabler;

pub struct DirectCopyEnabler;

impl Enabler for DirectCopyEnabler {
    fn activate(source: &Path, target: &Path) -> Result<(), ModManagerError> {
        todo!()
    }

    fn deactivate(target: &Path) -> Result<(), ModManagerError> {
        todo!()
    }
}
use crate::error::ModManagerError;
use std::path::Path;

pub trait Mod {
    fn id(&self) -> &str;
    fn install(&self, target: &Path) -> Result<(), ModManagerError>;
    fn validate(&self) -> Result<(), ModManagerError>;
}

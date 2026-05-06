use std::path::Path;
use crate::error::ModManagerError;

pub trait Mod {
    fn id(&self) -> &str;
    fn install(&self, target: &Path) -> Result<(), ModManagerError>;
    fn validate(&self) -> Result<(), ModManagerError>;
}

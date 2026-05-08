mod stardew;

pub use stardew::StardewValley;

use crate::config::Config;
use std::path::PathBuf;

pub trait Game {
    fn name(&self) -> &str;
    fn game_path(&self) -> PathBuf;
    fn mods_path(&self) -> PathBuf;
    fn staging_path(&self) -> PathBuf;
    fn discover_path(config: &Config) -> Option<PathBuf> {
        config
            .game_search_paths
            .get(Self::registry_id())?
            .iter()
            .find(|path| path.exists())
            .cloned()
    }
    fn registry_id() -> &'static str;
}

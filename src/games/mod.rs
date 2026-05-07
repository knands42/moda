mod stardew;

pub use stardew::StardewValley;

use std::path::PathBuf;
use crate::config::Config;

pub trait Game {
    fn name(&self) -> &str;
    fn game_path(&self) -> PathBuf;
    fn mods_path(&self) -> PathBuf;
    fn stock_path(&self) -> PathBuf;
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

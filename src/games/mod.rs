mod marvel_rivals;
mod stardew;

pub use marvel_rivals::MarvelRivals;
pub use stardew::StardewValley;

use crate::config::Config;
use crate::error::ModManagerError;
use std::path::PathBuf;

pub trait Game {
    fn name() -> &'static str;
    fn game_path(&self) -> PathBuf;
    fn set_game_path(&mut self, game_path: PathBuf);
    fn game_mod_path(&self) -> PathBuf;
    fn pre_setup(&self) -> Result<(), ModManagerError>;
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

pub fn supported_games() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            StardewValley::registry_id(),
            StardewValley::name(),
            "Farming simulator with deep modding support via SMAPI",
        ),
        (
            MarvelRivals::registry_id(),
            MarvelRivals::name(),
            "Hero shooter by NetEase — moddable via custom assets",
        ),
    ]
}

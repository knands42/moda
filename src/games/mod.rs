mod marvel_rivals;
mod stardew;
mod mad_max;

pub use marvel_rivals::MarvelRivals;
pub use stardew::StardewValley;
pub use mad_max::MadMax;

use crate::config::Config;
use crate::error::ModManagerError;
use crate::mods::{SyncManager, SyncManagerOps};
use std::path::PathBuf;

pub enum ModMode {
    Symlink,
    Pak,
    DirectCopy,
}

pub trait Game {
    fn descriptor(&self) -> &'static GameDescriptor;
    fn mod_mode(&self) -> ModMode;
    fn game_path(&self) -> PathBuf;
    fn set_game_path(&mut self, game_path: PathBuf);
    fn game_mod_path(&self) -> PathBuf;
    fn pre_setup(&self) -> Result<(), ModManagerError>;
}

pub struct GameDescriptor {
    pub name: &'static str,
    pub registry_id: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub construct: fn(PathBuf, Config) -> Box<dyn SyncManagerOps>,
}

impl GameDescriptor {
    pub fn discover_path(&self, config: &Config) -> Option<PathBuf> {
        config
            .game_search_paths
            .get(self.registry_id)?
            .iter()
            .find(|path| path.exists())
            .cloned()
    }
}

fn construct_stardew(path: PathBuf, config: Config) -> Box<dyn SyncManagerOps> {
    Box::new(SyncManager::new(StardewValley::new(path), config))
}

fn construct_marvel_rivals(path: PathBuf, config: Config) -> Box<dyn SyncManagerOps> {
    Box::new(SyncManager::new(MarvelRivals::new(path), config))
}

fn construct_mad_max(path: PathBuf, config: Config) -> Box<dyn SyncManagerOps> {
    Box::new(SyncManager::new(MadMax::new(path), config))
}

pub static STARDEW_VALLEY: GameDescriptor = GameDescriptor {
    name: "Stardew Valley",
    registry_id: "stardew_valley",
    description: "Farming simulator with deep modding support via SMAPI",
    icon: "\u{1F33E}",
    construct: construct_stardew,
};

pub static MARVEL_RIVALS: GameDescriptor = GameDescriptor {
    name: "Marvel Rivals",
    registry_id: "marvel_rivals",
    description: "Hero shooter by NetEase — bypass enabled",
    icon: "\u{2694}\u{FE0F}",
    construct: construct_marvel_rivals,
};

pub static MAD_MAX: GameDescriptor = GameDescriptor {
    name: "Mad Max: Fury Road",
    registry_id: "mad_max",
    description: "Open-world vehicular combat action-adventure set in a post-apocalyptic wasteland",
    icon: "\u{1F480}\u{1F525}\u{1F697}", // 💀🔥🚗
    construct: construct_mad_max,
};

pub fn registered_games() -> &'static [&'static GameDescriptor] {
    static GAMES: &[&GameDescriptor] = &[&STARDEW_VALLEY, &MAD_MAX];
    GAMES
}

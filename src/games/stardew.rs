use crate::games::Game;
use std::path::PathBuf;

pub struct StardewValley {
    game_path: PathBuf,
    mods_path: PathBuf,
    staging_path: PathBuf,
}

impl StardewValley {
    pub fn new(game_path: PathBuf, mods_path: PathBuf, staging_path: PathBuf) -> Self {
        Self {
            game_path,
            mods_path,
            staging_path,
        }
    }
}

impl Game for StardewValley {
    fn name(&self) -> &str {
        "Stardew Valley"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn mods_path(&self) -> PathBuf {
        self.mods_path.clone()
    }

    fn staging_path(&self) -> PathBuf {
        self.staging_path.clone()
    }

    fn registry_id() -> &'static str {
        // TODO: where to get/generate registry id
        "stardew_valley"
    }
}

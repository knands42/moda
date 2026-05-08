use std::fs;
use crate::games::Game;
use std::path::PathBuf;

pub struct StardewValley {
    game_path: PathBuf,
}

impl StardewValley {
    pub fn new(game_path: PathBuf) -> Self {
        let mods_path = game_path.join("Mods");

        if !mods_path.exists() {
            fs::create_dir_all(&mods_path).ok();
        }

        Self { game_path }
    }
}

impl Game for StardewValley {
    fn name(&self) -> &str {
        "Stardew Valley"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.join("Mods")
    }

    fn registry_id() -> &'static str {
        // TODO: where to get/generate registry id
        "stardew_valley"
    }
}

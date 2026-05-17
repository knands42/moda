use crate::error::ModManagerError;
use crate::games::Game;
use std::fs;
use std::path::PathBuf;

pub struct StardewValley {
    game_path: PathBuf,
}

impl StardewValley {
    pub fn new(game_path: PathBuf) -> Self {
        let mods_path = game_path.join("Mods");

        if !mods_path.exists() {
            fs::create_dir_all(&mods_path).ok();
            log::info!("Created Mods folder at {}", mods_path.display());
        }

        log::info!("Stardew Valley initialized at {}", game_path.display());
        Self { game_path }
    }
}

impl Game for StardewValley {
    fn name() -> &'static str {
        "Stardew Valley"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.join("Mods")
    }

    fn pre_setup(&self) -> Result<(), ModManagerError> {
        todo!()
    }

    fn registry_id() -> &'static str {
        "stardew_valley"
    }
}

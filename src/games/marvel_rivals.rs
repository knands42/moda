use crate::error::ModManagerError;
use crate::games::Game;
use std::path::PathBuf;

pub struct MarvelRivals {
    game_path: PathBuf,
}

impl MarvelRivals {
    pub fn new(game_path: PathBuf) -> Self {
        let mods_path = game_path.join("Mods");
        if !mods_path.exists() {
            std::fs::create_dir_all(&mods_path).ok();
        }
        log::info!("Marvel Rivals initialized at {}", game_path.display());
        Self { game_path }
    }
}

impl Game for MarvelRivals {
    fn name() -> &'static str {
        "Marvel Rivals"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn set_game_path(&mut self, game_path: PathBuf) {
        self.game_path = game_path;
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.join("Mods")
    }

    fn pre_setup(&self) -> Result<(), ModManagerError> {
        log::info!("No pre-setup needed for Marvel Rivals");
        Ok(())
    }

    fn registry_id() -> &'static str {
        "marvel_rivals"
    }
}

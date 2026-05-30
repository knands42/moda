use crate::error::ModManagerError;
use crate::games::{Game, GameDescriptor, ModMode};
use std::fs;
use std::path::PathBuf;

pub struct MadMax {
    game_path: PathBuf,
}

impl MadMax {
    pub fn new(game_path: PathBuf) -> Self {
        let max_max = Self {
            game_path: game_path.clone(),
        };

        let mods_path = max_max.game_mod_path();
        if !mods_path.exists() {
            fs::create_dir_all(&mods_path).ok();
            log::info!("Created Mods folder at {}", mods_path.display());
        }

        log::info!("Mad Max initialized at {}", game_path.display());
        max_max
    }
}

impl Game for MadMax {
    fn descriptor(&self) -> &'static GameDescriptor {
        &crate::games::MAD_MAX
    }

    fn mod_mode(&self) -> ModMode {
        ModMode::Symlink
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn set_game_path(&mut self, game_path: PathBuf) {
        self.game_path = game_path;
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn pre_setup(&self) -> Result<(), ModManagerError> {
        Ok(())
    }
}

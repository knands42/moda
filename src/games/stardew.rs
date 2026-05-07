use std::collections::HashMap;
use std::path::PathBuf;
use crate::games::config::Config;
use crate::games::Game;

pub struct StardewValley {
    game_path: PathBuf,
    mods_path: PathBuf,
    stock_path: PathBuf,
}

impl StardewValley {
    pub fn new(game_path: PathBuf, mods_path: PathBuf, stock_path: PathBuf) -> Option<Self> {
        Some(Self {
            game_path,
            mods_path,
            stock_path,
        })
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
    
    fn registry_id() -> &'static str {
        // TODO: where to get/generate registry id
        "stardew_valley"
    }
}


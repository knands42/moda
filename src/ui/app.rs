use std::path::PathBuf;

use crate::config::Config;
use crate::games::{Game, StardewValley};
use crate::mods::ModState;

use super::message::Tab;

pub struct App {
    pub game_name: String,
    pub game_path: String,
    pub mods_path: String,
    pub staging_path: String,
    pub game_mod_path: String,
    pub mod_state: ModState,
    pub log: Vec<String>,
    pub current_tab: Tab,
}

impl Default for App {
    fn default() -> Self {
        let (game_path, mods_path, staging_path, game_mod_path) = match Config::load_config() {
            Some(config) => match StardewValley::discover_path(&config) {
                Some(gp) => {
                    let mods =
                        PathBuf::from(&config.mods_root_path).join(StardewValley::registry_id());
                    let staging =
                        PathBuf::from(&config.staging_root_path).join(StardewValley::registry_id());
                    (
                        gp.to_string_lossy().to_string(),
                        mods.to_string_lossy().to_string(),
                        staging.to_string_lossy().to_string(),
                        gp.join("Mods").to_string_lossy().to_string(),
                    )
                }
                None => (
                    "Not found".into(),
                    config.mods_root_path.clone(),
                    config.staging_root_path.clone(),
                    "N/A".into(),
                ),
            },
            None => (
                "Config not loaded".into(),
                String::new(),
                String::new(),
                String::new(),
            ),
        };

        App {
            game_name: "Stardew Valley".into(),
            game_path,
            mods_path,
            staging_path,
            game_mod_path,
            mod_state: ModState::default(),
            log: vec!["Ready. Click Reconcile to scan your mods.".into()],
            current_tab: Tab::Mods,
        }
    }
}

impl App {
    pub fn push_log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }
}

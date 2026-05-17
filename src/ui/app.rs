use crate::mods::{ModState, SyncManagerOps};

use super::message::Tab;

pub struct App {
    pub game_name: String,
    pub game_path: String,
    pub mods_path: String,
    pub staging_path: String,
    pub game_mod_path: String,
    pub mod_state: ModState,
    pub sync_manager: Option<Box<dyn SyncManagerOps>>,
    pub log: Vec<String>,
    pub current_tab: Tab,
}

impl Default for App {
    fn default() -> Self {
        App {
            game_name: String::new(),
            game_path: String::new(),
            mods_path: String::new(),
            staging_path: String::new(),
            game_mod_path: String::new(),
            mod_state: ModState::default(),
            sync_manager: None,
            log: vec!["Select a game to get started.".into()],
            current_tab: Tab::GameSelect,
        }
    }
}

impl App {
    pub fn push_log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }
}

use std::path::PathBuf;

use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::{Game, GameDescriptor, ModMode};
use crate::mods::catalog::ModEntry;
use crate::mods::ModState;
use crate::mods::SyncManagerOps;

pub struct ActiveGame {
    sync_manager: Box<dyn SyncManagerOps>,
    state: ModState,
}

impl ActiveGame {
    pub fn create(
        descriptor: &'static GameDescriptor,
        path: PathBuf,
        config: &Config,
    ) -> Result<Self, ModManagerError> {
        let sync_manager = (descriptor.construct)(path, config.clone());
        let state = sync_manager.reconcile()?;
        Ok(Self {
            sync_manager,
            state,
        })
    }

    pub fn game(&self) -> &dyn Game {
        self.sync_manager.game()
    }

    pub fn descriptor(&self) -> &'static GameDescriptor {
        self.game().descriptor()
    }

    pub fn registry_id(&self) -> &'static str {
        self.descriptor().registry_id
    }

    pub fn mod_mode(&self) -> ModMode {
        self.game().mod_mode()
    }

    pub fn game_name(&self) -> &'static str {
        self.descriptor().name
    }

    pub fn game_mod_path(&self) -> PathBuf {
        self.sync_manager.game_mod_path()
    }

    pub fn sync_all(&mut self) -> Result<(), ModManagerError> {
        self.sync_manager.sync_all(&mut self.state)
    }

    pub fn reconcile(&mut self) -> Result<(), ModManagerError> {
        self.state = self.sync_manager.reconcile()?;
        Ok(())
    }

    pub fn stage_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        self.sync_manager.stage_one_mod(mod_entry, &mut self.state)
    }

    pub fn unstage_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        self.sync_manager.unstage_one_mod(mod_entry, &mut self.state)
    }

    pub fn enable_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        self.sync_manager.enable_one_mod(mod_entry, &mut self.state)
    }

    pub fn disable_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        self.sync_manager.disable_one_mod(mod_entry, &mut self.state)
    }

    pub fn state(&self) -> &ModState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ModState {
        &mut self.state
    }
}

use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::catalog::ModEntry;
use crate::mods::ModState;
use std::path::PathBuf;

pub mod sync_manager;
pub use sync_manager::SyncManager;

pub trait SyncManagerOps {
    fn game(&self) -> &dyn Game;
    fn game_mod_path(&self) -> PathBuf;
    fn reconcile(&self) -> Result<ModState, ModManagerError>;
    fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError>;
    fn stage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError>;
    fn unstage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError>;
    fn enable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError>;
    fn disable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError>;
}

impl<G: Game> SyncManagerOps for SyncManager<G> {
    fn game(&self) -> &dyn Game {
        &self.game
    }

    fn game_mod_path(&self) -> PathBuf {
        SyncManager::game_mod_path(self)
    }

    fn reconcile(&self) -> Result<ModState, ModManagerError> {
        SyncManager::reconcile(self, &SyncManager::game_mod_path(self))
    }

    fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        SyncManager::sync_all(self, state)
    }

    fn stage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        SyncManager::stage_one_mod(self, mod_entry, state)
    }

    fn unstage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        SyncManager::unstage_one_mod(self, mod_entry, state)
    }

    fn enable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        SyncManager::enable_one_mod(self, mod_entry, state)
    }

    fn disable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        SyncManager::disable_one_mod(self, mod_entry, state)
    }
}

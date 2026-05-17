use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::catalog::ModEntry;
use crate::mods::ModState;
use std::path::Path;

pub mod sync_manager;
pub use sync_manager::SyncManager;

pub trait SyncManagerOps {
    fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError>;
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
    fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        SyncManager::reconcile(self, game_mod_path)
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

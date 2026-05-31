mod db;
mod mod_repo;
mod profile_repo;

use crate::error::ModManagerError;
use crate::mods::ModState;
use crate::mods::types::ReconciledMod;

pub trait ModRepository: Send + Sync {
    /// Read all reconciled mods for a game (SSOT for game entries)
    fn get_mods(&self, game_registry_id: &str) -> Result<ModState, ModManagerError>;

    /// Upsert after mutation (status, paths change)
    fn upsert_mod(&self, game_registry_id: &str, mood: &ReconciledMod) -> Result<(), ModManagerError>;

    /// Remove a mod
    fn remove_mod(&self, game_registry_id: &str, name: &str) -> Result<(), ModManagerError>;
}
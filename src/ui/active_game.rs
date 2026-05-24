use std::path::PathBuf;

use crate::error::ModManagerError;
use crate::games::{Game, MarvelRivals, StardewValley};
use crate::mods::catalog::ModEntry;
use crate::mods::ModState;
use crate::mods::SyncManager;

// TODO: make this not rely on additional entries, but dynamic
pub enum ActiveGame {
    StardewValley(SyncManager<StardewValley>, ModState),
    MarvelRivals(SyncManager<MarvelRivals>, ModState),
}

impl ActiveGame {
    pub fn registry_id(&self) -> &'static str {
        match self {
            ActiveGame::StardewValley(..) => StardewValley::registry_id(),
            ActiveGame::MarvelRivals(..) => MarvelRivals::registry_id(),
        }
    }

    pub fn game_name(&self) -> &'static str {
        match self {
            ActiveGame::StardewValley(..) => StardewValley::name(),
            ActiveGame::MarvelRivals(..) => MarvelRivals::name(),
        }
    }

    pub fn game_mod_path(&self) -> PathBuf {
        match self {
            ActiveGame::StardewValley(sm, _) => sm.game_mod_path(),
            ActiveGame::MarvelRivals(sm, _) => sm.game_mod_path(),
        }
    }

    pub fn sync_all(&mut self) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => sm.sync_all(state),
            ActiveGame::MarvelRivals(sm, state) => sm.sync_all(state),
        }
    }

    pub fn reconcile(&mut self) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => {
                *state = sm.reconcile(&sm.game_mod_path())?;
                Ok(())
            }
            ActiveGame::MarvelRivals(sm, state) => {
                *state = sm.reconcile(&sm.game_mod_path())?;
                Ok(())
            }
        }
    }

    pub fn stage_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => sm.stage_one_mod(mod_entry, state),
            ActiveGame::MarvelRivals(sm, state) => sm.stage_one_mod(mod_entry, state),
        }
    }

    pub fn unstage_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => sm.unstage_one_mod(mod_entry, state),
            ActiveGame::MarvelRivals(sm, state) => sm.unstage_one_mod(mod_entry, state),
        }
    }

    pub fn enable_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => sm.enable_one_mod(mod_entry, state),
            ActiveGame::MarvelRivals(sm, state) => sm.enable_one_mod(mod_entry, state),
        }
    }

    pub fn disable_one_mod(&mut self, mod_entry: &ModEntry) -> Result<(), ModManagerError> {
        match self {
            ActiveGame::StardewValley(sm, state) => sm.disable_one_mod(mod_entry, state),
            ActiveGame::MarvelRivals(sm, state) => sm.disable_one_mod(mod_entry, state),
        }
    }

    pub fn state(&self) -> &ModState {
        match self {
            ActiveGame::StardewValley(_, state) => state,
            ActiveGame::MarvelRivals(_, state) => state,
        }
    }

    pub fn state_mut(&mut self) -> &mut ModState {
        match self {
            ActiveGame::StardewValley(_, state) => state,
            ActiveGame::MarvelRivals(_, state) => state,
        }
    }
}

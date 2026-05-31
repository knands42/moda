use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::catalog::Catalog;
use crate::mods::enabler::Enabler;
use crate::mods::mod_state::ModState;
use crate::mods::stager::Stager;
use crate::mods::types::ModEntry;
use crate::mods::types::ModStatus;
use crate::mods::{DirectCopyStager, SymlinkEnabler};
use std::path::{Path, PathBuf};

pub struct SyncManager<G: Game> {
    pub(super) game: G,
    config: Config,
    catalog: Catalog,
}

impl<G: Game> SyncManager<G> {
    pub fn new(game: G, config: Config) -> Self {
        let descriptor = game.descriptor();
        let catalog = Catalog::new(config.clone(), descriptor.registry_id);
        log::debug!("SyncManager created for game: {}", descriptor.name);
        Self {
            game,
            config,
            catalog,
        }
    }
}

impl<G: Game> SyncManager<G> {
    pub fn stage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        log::info!("Staging mod: {}", mod_entry.name);
        if let Some(staging_entry) = mod_entry.kind.stage(mod_entry, &staging_path)? {
            state.set_staged(&staging_entry);
        }
        Ok(())
    }

    pub fn unstage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        log::info!("Unstaging mod: {}", mod_entry.name);
        let _ = SymlinkEnabler::deactivate(&self.game.game_mod_path().join(&mod_entry.name));

        if mod_entry.path.exists() {
            DirectCopyStager::unstage(&mod_entry.path)?;
        }

        self.resolve_after_unstage(mod_entry, state);
        Ok(())
    }

    fn stage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Staging all mods");
        for m in state.snapshot() {
            if m.status == ModStatus::Downloaded {
                if let Some(downloaded_mod) = m.source_entry.as_ref() {
                    self.stage_one_mod(downloaded_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a source folder", m.name);
                }
            }
        }

        Ok(())
    }
}

impl<G: Game> SyncManager<G> {
    pub fn enable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        log::info!("Enabling mod: {}", mod_entry.name);
        let game_entry = SymlinkEnabler::enable(mod_entry, &self.game.game_mod_path())?;
        state.set_enabled(&game_entry);
        Ok(())
    }

    pub fn disable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        log::info!("Disabling mod: {}", mod_entry.name);
        if mod_entry.path.exists() {
            SymlinkEnabler::disable(mod_entry)?;
        }
        self.resolve_after_disable(mod_entry, state)?;
        Ok(())
    }

    pub fn unstage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Unstaging all mods");

        for m in state.snapshot() {
            if m.status == ModStatus::Staged || m.status == ModStatus::Enabled {
                if let Some(staged_mod) = m.staging_entry.as_ref() {
                    self.unstage_one_mod(staged_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a staging folder", m.name);
                }
            }
        }

        Ok(())
    }

    pub fn disable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Disabling all enabled mods");

        for m in state.snapshot() {
            if m.status == ModStatus::Enabled {
                if let Some(staged_mod) = m.game_entry.as_ref() {
                    self.disable_one_mod(staged_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a staging folder", m.name);
                }
            }
        }

        Ok(())
    }

    fn enable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Enabling all staged mods");

        for m in state.snapshot() {
            if m.status == ModStatus::Staged {
                if let Some(staged_mod) = m.staging_entry.as_ref() {
                    self.enable_one_mod(staged_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a staging folder", m.name);
                }
            }
        }

        Ok(())
    }
}

impl<G: Game> SyncManager<G> {
    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        log::info!("Reconciling mod state from {}", game_mod_path.display());
        let state = self.catalog.reconcile(game_mod_path)?;
        let count = state.snapshot().len();
        log::info!("Reconcile complete: {} mods found", count);
        Ok(state)
    }

    pub fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Sync all started");

        self.stage_mods(state)?;
        self.enable_mods(state)?;

        log::info!("Sync all complete");
        Ok(())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(self.game.descriptor().registry_id)
    }
}

impl<G: Game> SyncManager<G> {
    pub fn game_mod_path(&self) -> PathBuf {
        self.game.game_mod_path()
    }

    pub fn game_path(&self) -> PathBuf {
        self.game.game_path()
    }
}

impl<G: Game> SyncManager<G> {
    fn resolve_after_disable(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        let Some(reconciled) = state.get_mod(&mod_entry.name) else {
            return Ok(());
        };

        if reconciled
            .staging_entry
            .as_ref()
            .is_some_and(|e| e.path.exists())
        {
            state.set_disabled(&mod_entry.name);
        } else if reconciled
            .source_entry
            .as_ref()
            .is_some_and(|e| e.path.exists())
        {
            state.set_unstaged(&mod_entry.name);
        } else {
            state.remove(&mod_entry.name);
        }

        Ok(())
    }

    fn resolve_after_unstage(&self, mod_entry: &ModEntry, state: &mut ModState) {
        state.set_unstaged(&mod_entry.name);
        if state
            .get_mod(&mod_entry.name)
            .and_then(|m| m.source_entry.clone())
            .is_some()
        {
            state.set_downloaded(mod_entry);
        } else {
            state.remove(&mod_entry.name);
        }
    }
}

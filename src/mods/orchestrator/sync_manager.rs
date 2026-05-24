use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::{Game, ModMode};
use crate::mods::catalog::{Catalog, ModEntry, ModEntryKind, ModStatus};
use crate::mods::installer::strip_zip_ext;
use crate::mods::mod_state::ModState;
use crate::mods::{SymlinkEnabler, Installer, ModSource};
use std::path::{Path, PathBuf};

pub struct SyncManager<G: Game> {
    pub(super) game: G,
    config: Config,
    mod_registry: Catalog,
}

impl<G: Game> SyncManager<G> {
    pub fn new(game: G, config: Config) -> Self {
        let descriptor = game.descriptor();
        let mod_registry = Catalog::new(config.clone(), descriptor.registry_id);
        log::debug!("SyncManager created for game: {}", descriptor.name);
        Self {
            game,
            config,
            mod_registry,
        }
    }
}

impl<G: Game> SyncManager<G> {
    pub fn stage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Staging all mods");
        for m in state.snapshot() {
            if m.status == ModStatus::Downloaded {
                if let Some(downloaded_mod) = m.source_entry.as_ref() {
                    self.stage_one_mod(downloaded_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a source folder", m.name);
                    // TODO: reconcile?
                }
            }
        }

        Ok(())
    }

    pub fn stage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        log::info!("Staging mod: {}", mod_entry.name);
        match mod_entry.kind {
            ModEntryKind::Directory => {
                let target = staging_path.join(&mod_entry.name);
                Installer::install(&ModSource::LocalDir(mod_entry.path.clone()), &target)?;
                let staging_entry = ModEntry {
                    name: mod_entry.name.clone(),
                    path: target,
                    kind: ModEntryKind::Directory,
                    metadata: None,
                };
                state.set_staged(&staging_entry);
            }
            ModEntryKind::ZipArchive => {
                let (staging_name, target) = match Installer::zip_wrap_directory(&mod_entry.path)? {
                    Some(dir) => (dir, staging_path.clone()),
                    None => {
                        let name = strip_zip_ext(&mod_entry.name);
                        (name.clone(), staging_path.join(&name))
                    }
                };
                Installer::install(&ModSource::LocalZip(mod_entry.path.clone()), &target)?;
                let staging_entry = ModEntry {
                    name: staging_name.clone(),
                    path: staging_path.join(&staging_name),
                    kind: ModEntryKind::Directory,
                    metadata: None,
                };
                state.set_staged(&staging_entry);
            }
            _ => {}
        }

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
                    // TODO: reconcile?
                }
            }
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
            Installer::uninstall_from_dir(&mod_entry.path)?;
        }

        self.resolve_after_unstage(mod_entry, state);
        Ok(())
    }
}

impl<G: Game> SyncManager<G> {
    pub fn enable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Enabling all staged mods");

        for m in state.snapshot() {
            if m.status == ModStatus::Staged {
                if let Some(staged_mod) = m.staging_entry.as_ref() {
                    self.enable_one_mod(staged_mod, state)?
                } else {
                    log::warn!("Mod {} doesnt have a staging folder", m.name);
                    // TODO: reconcile?
                }
            }
        }

        Ok(())
    }

    pub fn enable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        log::info!("Enabling mod: {}", mod_entry.name);
        let game_mods_path = self.game.game_mod_path();
        let game_entry_path = game_mods_path.join(&mod_entry.name);
        SymlinkEnabler::activate(mod_entry.path.as_path(), game_entry_path.as_path())?;

        let game_entry = ModEntry {
            name: mod_entry.name.clone(),
            path: game_entry_path,
            kind: ModEntryKind::Directory,
            metadata: None,
        };
        state.set_enabled(&game_entry);
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
                    // TODO: reconcile?
                }
            }
        }

        Ok(())
    }

    pub fn disable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        log::info!("Disabling mod: {}", mod_entry.name);
        if mod_entry.path.exists() {
            SymlinkEnabler::deactivate(&mod_entry.path)?;
        }

        self.resolve_after_disable(mod_entry, state)?;
        Ok(())
    }
}

impl<G: Game> SyncManager<G> {
    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        log::info!("Reconciling mod state from {}", game_mod_path.display());
        let state = self.mod_registry.reconcile(game_mod_path)?;
        let count = state.snapshot().len();
        log::info!("Reconcile complete: {} mods found", count);
        Ok(state)
    }

    // TODO: Make it handle Modified
    pub fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Sync all started");
        let mut staged = 0;
        let mut enabled = 0;

        for m in state.snapshot() {
            if m.status == ModStatus::Downloaded {
                self.stage_one_mod(m.source_entry.as_ref().unwrap(), state)?;
                staged += 1;
            }
        }

        for m in state.snapshot() {
            if m.status == ModStatus::Staged {
                self.enable_one_mod(m.staging_entry.as_ref().unwrap(), state)?;
                enabled += 1;
            }
        }

        log::info!("Sync all complete: staged={staged}, enabled={enabled}");
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

use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::installer::strip_zip_ext;
use crate::mods::mod_registry::{ModEntry, ModEntryKind, ModRegistry, ModStatus};
use crate::mods::mod_state::ModState;
use crate::mods::{Enabler, Installer, ModSource};
use std::path::{Path, PathBuf};

pub struct SyncManager<G: Game> {
    game: G,
    config: Config,
    mod_registry: ModRegistry<G>,
}

impl<G: Game> SyncManager<G> {
    pub fn new(game: G, config: Config) -> Self {
        let mod_registry = ModRegistry::new(config.clone());
        Self {
            game,
            config,
            mod_registry,
        }
    }

    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        self.mod_registry.reconcile(game_mod_path)
    }

    pub fn stage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        self.disable_mods(state)?;

        let mods_folder = self.mod_registry.list_mods_folder()?;

        for entry in mods_folder {
            self.stage_one_mod(&entry, state)?;
        }

        Ok(())
    }

    pub fn stage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        match mod_entry.kind {
            ModEntryKind::Directory => {
                let target = staging_path.join(&mod_entry.name);
                Installer::install(&ModSource::LocalDir(mod_entry.path.clone()), &target)?;
            }
            ModEntryKind::ZipArchive => {
                let target = match Installer::zip_wrap_directory(&mod_entry.path)? {
                    Some(_) => staging_path,
                    None => staging_path.join(strip_zip_ext(&mod_entry.name)),
                };
                Installer::install(&ModSource::LocalZip(mod_entry.path.clone()), &target)?;
            }
            _ => {}
        }

        state.set_staged(&mod_entry.name);
        Ok(())
    }

    pub fn unstage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        let staging_path = self.mod_registry.list_staging_folder()?;
        for entry in staging_path {
            self.unstage_one_mod(&entry, state)?;
        }

        Ok(())
    }

    pub fn unstage_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        if mod_entry.path.exists() {
            Installer::uninstall_from_dir(&mod_entry.path)?;
        }

        match self.mod_registry.get_mod_by_name(&mod_entry.name) {
            Ok(_) => {
                if state
                    .mods
                    .iter_mut()
                    .find(|m| m.name == mod_entry.name)
                    .is_some()
                {
                    state.set_unstaged(&mod_entry.name);
                } else {
                    self.mod_registry
                        .reconcile(self.game.game_mod_path().as_path())?;
                }
                Ok(())
            }
            Err(ModManagerError::ModNotFound(_)) => {
                state.remove(&mod_entry.name);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn enable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        let staging_path = self.mod_registry.list_staging_folder()?;
        for entry in staging_path {
            self.enable_one_mod(&entry, state)?;
        }

        Ok(())
    }

    pub fn enable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        let game_mods_path = self.game.game_mod_path();
        Enabler::activate(
            mod_entry.path.as_path(),
            game_mods_path.join(&mod_entry.name).as_path(),
        )?;

        state.set_enabled(&mod_entry.name);
        Ok(())
    }

    pub fn disable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        let game_mods_path = self.game.game_mod_path();
        let game_mods = self.mod_registry.list_game_mods_folder(&game_mods_path)?;
        for entry in game_mods {
            self.disable_one_mod(&entry, state)?;
        }

        Ok(())
    }

    pub fn disable_one_mod(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        if mod_entry.path.exists() {
            Enabler::deactivate(&mod_entry.path)?;
        }

        match self.mod_registry.get_staged_mod_by_name(&mod_entry.name) {
            Ok(_) => {
                state.set_disabled(&mod_entry.name);
                Ok(())
            }
            Err(ModManagerError::ModNotFound(_)) => self.unstage_one_mod(mod_entry, state),
            Err(e) => Err(e),
        }
    }

    pub fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        let reconciled = state.mods.clone();
        for m in &reconciled {
            match m.status {
                ModStatus::Downloaded => {
                    self.stage_one_mod(m.source_entry.as_ref().unwrap(), state)?
                }
                ModStatus::Staged => {
                    self.enable_one_mod(m.staging_entry.as_ref().unwrap(), state)?
                }
                ModStatus::Enabled => continue,
                ModStatus::Modified => todo!(),
            }
        }

        Ok(())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(G::registry_id())
    }
}

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

pub enum ResolveStatusAfterDisable { Staged, Downloaded, NotFound }

pub enum ResolveStatusAfterUnstage { Downloaded, NotFound }

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

    // TODO: what to do with new updated mods, will it always disabled first, stage and then re-enabled?
    pub fn stage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
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
                state.set_staged(&mod_entry.name);
            }
            ModEntryKind::ZipArchive => {
                let (staging_name, target) = match Installer::zip_wrap_directory(&mod_entry.path)? {
                    Some(dir) => (dir, staging_path),
                    None => {
                        let name = strip_zip_ext(&mod_entry.name);
                        let target = staging_path.join(&name);
                        (name, target)
                    }
                };
                Installer::install(&ModSource::LocalZip(mod_entry.path.clone()), &target)?;
                state.set_staged(&staging_name);
            }
            _ => {},
        }

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
        let _ = Enabler::deactivate(&self.game.game_mod_path().join(&mod_entry.name));

        if mod_entry.path.exists() {
            Installer::uninstall_from_dir(&mod_entry.path)?;
        }

        match self.resolve_status_after_unstage(&mod_entry.name)? {
            ResolveStatusAfterUnstage::Downloaded => state.set_downloaded(&mod_entry.name),
            ResolveStatusAfterUnstage::NotFound => state.remove(&mod_entry.name),
        }
        Ok(())
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

        match self.resolve_status_after_disable(&mod_entry.name)? {
            ResolveStatusAfterDisable::Staged => state.set_staged(&mod_entry.name),
            ResolveStatusAfterDisable::Downloaded => state.set_downloaded(&mod_entry.name),
            ResolveStatusAfterDisable::NotFound => state.remove(&mod_entry.name),
        }
        Ok(())
    }

    pub fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        let reconciled = state.snapshot();
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

    fn resolve_status_after_disable(
        &self,
        mod_name: &str,
    ) -> Result<ResolveStatusAfterDisable, ModManagerError> {
        if self.mod_registry.get_staged_mod_by_name(mod_name).is_ok() {
            return Ok(ResolveStatusAfterDisable::Staged);
        }
        if self.mod_registry.get_mod_by_name(mod_name).is_ok() {
            return Ok(ResolveStatusAfterDisable::Downloaded);
        }
        Ok(ResolveStatusAfterDisable::NotFound) // not found anywhere → remove
    }

    fn resolve_status_after_unstage(
        &self,
        mod_name: &str,
    ) -> Result<ResolveStatusAfterUnstage, ModManagerError> {
        if self.mod_registry.get_staged_mod_by_name(mod_name).is_ok() {
            return Ok(ResolveStatusAfterUnstage::Downloaded)
        }

        Ok(ResolveStatusAfterUnstage::NotFound)
    }
}

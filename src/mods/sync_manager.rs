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

// TODO: split into multiple impl blocks
impl<G: Game> SyncManager<G> {
    pub fn new(game: G, config: Config) -> Self {
        let mod_registry = ModRegistry::new(config.clone());
        log::debug!("SyncManager created for game: {}", game.name());
        Self {
            game,
            config,
            mod_registry,
        }
    }

    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        log::info!("Reconciling mod state from {}", game_mod_path.display());
        let state = self.mod_registry.reconcile(game_mod_path)?;
        let count = state.snapshot().len();
        log::info!("Reconcile complete: {} mods found", count);
        Ok(state)
    }

    // TODO: what to do with new updated mods, will it always disabled first, stage and then re-enabled?
    pub fn stage_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Staging all mods");
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
        log::info!("Unstaging mod: {}", mod_entry.name);
        let _ = Enabler::deactivate(&self.game.game_mod_path().join(&mod_entry.name));

        if mod_entry.path.exists() {
            Installer::uninstall_from_dir(&mod_entry.path)?;
        }

        self.resolve_after_unstage(mod_entry, state);
        Ok(())
    }

    pub fn enable_mods(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Enabling all staged mods");
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
        log::info!("Enabling mod: {}", mod_entry.name);
        let game_mods_path = self.game.game_mod_path();
        let game_entry_path = game_mods_path.join(&mod_entry.name);
        Enabler::activate(mod_entry.path.as_path(), game_entry_path.as_path())?;

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
        log::info!("Disabling mod: {}", mod_entry.name);
        if mod_entry.path.exists() {
            Enabler::deactivate(&mod_entry.path)?;
        }

        self.resolve_after_disable(mod_entry, state)?;
        Ok(())
    }

    // TODO: Make it handle Modified
    // TODO: Goes from Downloaded to Enabled
    pub fn sync_all(&self, state: &mut ModState) -> Result<(), ModManagerError> {
        log::info!("Sync all started");
        let reconciled = state.snapshot();
        let mut staged = 0;
        let mut enabled = 0;
        for m in &reconciled {
            match m.status {
                ModStatus::Downloaded => {
                    self.stage_one_mod(m.source_entry.as_ref().unwrap(), state)?;
                    staged += 1;
                }
                ModStatus::Staged => {
                    self.enable_one_mod(m.staging_entry.as_ref().unwrap(), state)?;
                    enabled += 1;
                }
                ModStatus::Enabled => continue,
                ModStatus::Modified => todo!(),
            }
        }

        log::info!("Sync all complete: staged={staged}, enabled={enabled}");
        Ok(())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(G::registry_id())
    }

    fn resolve_after_disable(
        &self,
        mod_entry: &ModEntry,
        state: &mut ModState,
    ) -> Result<(), ModManagerError> {
        state.set_disabled(&mod_entry.name);
        if self
            .mod_registry
            .get_staged_mod_by_name(&mod_entry.name)
            .is_ok()
        {
            let staging = self.mod_registry.get_staged_mod_by_name(&mod_entry.name)?;
            state.set_staged(&staging);
        } else if self.mod_registry.get_mod_by_name(&mod_entry.name).is_ok() {
            let src = self.mod_registry.get_mod_by_name(&mod_entry.name)?;
            state.set_downloaded(&src);
        } else {
            state.remove(&mod_entry.name);
        }

        Ok(())
    }

    fn resolve_after_unstage(&self, mod_entry: &ModEntry, state: &mut ModState) {
        state.set_unstaged(&mod_entry.name);
        if let Ok(src) = self.mod_registry.get_mod_by_name(&mod_entry.name) {
            state.set_downloaded(&src);
        } else {
            state.remove(&mod_entry.name);
        }
    }
}

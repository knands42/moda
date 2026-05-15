use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::mod_registry::{ModEntry, ModEntryKind, ModRegistry};
use crate::mods::{Enabler, Installer, ModSource};
use std::path::PathBuf;

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

    pub fn stage_mods(&self) -> Result<(), ModManagerError> {
        let mods_folder = self.mod_registry.list_mods_folder()?;

        for entry in mods_folder {
            self.stage_one_mod(entry)?;
        }

        Ok(())
    }

    pub fn stage_one_mod(&self, mod_entry: ModEntry) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        if mod_entry.kind == ModEntryKind::Directory {
            Installer::install(&ModSource::LocalDir(mod_entry.path), staging_path.as_path())?;
        } else if mod_entry.kind == ModEntryKind::ZipArchive {
            Installer::install(&ModSource::LocalZip(mod_entry.path), staging_path.as_path())?;
        }

        Ok(())
    }

    pub fn enable_mods(&self) -> Result<(), ModManagerError> {
        let staging_path = self.mod_registry.list_staging_folder()?;
        for entry in staging_path {
            self.enable_one_mod(entry)?;
        }

        Ok(())
    }

    pub fn enable_one_mod(&self, mod_entry: ModEntry) -> Result<(), ModManagerError> {
        let game_mods_path = self.game.game_mod_path();
        Enabler::activate(
            mod_entry.path.as_path(),
            game_mods_path.join(mod_entry.name).as_path(),
        )?;

        Ok(())
    }

    pub fn sync_all(&self) -> Result<(), ModManagerError> {
        self.stage_mods()?;
        self.enable_mods()?;

        Ok(())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(G::registry_id())
    }
}

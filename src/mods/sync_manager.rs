use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::{Enabler, Installer, ModSource};

pub struct SyncManager<G: Game> {
    game: G,
    config: Config
}

impl<G: Game> SyncManager<G> {
    pub fn new(game: G, config: Config) -> Self {
        Self { game, config }
    }

    pub fn stage_mods(&self) -> Result<(), ModManagerError> {
        for entry in fs::read_dir(self.get_mod_path())? {
            let source_path = entry?;
            self.stage_one_mod(source_path)?;
        }

        Ok(())
    }

    pub fn stage_one_mod(&self, source_path: DirEntry) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        if source_path.file_type()?.is_dir() {
            Installer::install(&ModSource::LocalDir(source_path.path()), staging_path.as_path())?;

        } else if source_path.path().extension().map_or(false, |ext| ext == "zip") {
            Installer::install(&ModSource::LocalZip(source_path.path()), staging_path.as_path())?;
        }

        Ok(())
    }

    pub fn enable_mods(&self) -> Result<(), ModManagerError> {
        let staging_path = self.get_staging_path();
        for entry in fs::read_dir(staging_path)? {
            let source_path = entry?;
            self.enable_one_mod(source_path)?;
        }

        Ok(())
    }

    pub fn enable_one_mod(&self, source_path: DirEntry) -> Result<(), ModManagerError> {
        let game_mods_path = self.game.game_mod_path();
        Enabler::activate(source_path.path().as_path(), game_mods_path.as_path())?;

        Ok(())
    }

    pub fn sync_all(&self) -> Result<(), ModManagerError> {
        self.stage_mods()?;
        self.enable_mods()?;

        Ok(())
    }

    fn get_mod_path(&self) -> PathBuf {
        PathBuf::from(&self.config.mods_root_path).join(G::registry_id())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(G::registry_id())
    }
}
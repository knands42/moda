use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::fs;

pub struct ModEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: ModEntryKind,
}

pub enum ModStatus {
    Downloaded,
    Staged,
    Enabled,
    Modified
}

pub struct ReconciledMod {
    pub name: String,
    pub status: ModStatus,
    pub source_entry: Option<ModEntry>,
    pub staging_entry: Option<ModEntry>,
    pub game_entry: Option<ModEntry>,
}

#[derive(Debug, PartialEq)]
pub enum ModEntryKind {
    Directory,
    ZipArchive,
    Other,
}

pub struct ModRegistry<G: Game> {
    config: Config,
    _game: PhantomData<G>,
}

impl<G: Game> ModRegistry<G> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            _game: PhantomData,
        }
    }
}

impl<G: Game> ModRegistry<G> {
    pub fn list_mods_folder(&self) -> Result<Vec<ModEntry>, ModManagerError> {
        let mods_root_path = self.get_mod_path();
        self.list_folder(mods_root_path)
    }

    pub fn get_mod_by_name(&self, name: &str) -> Result<ModEntry, ModManagerError> {
        let mods_root_path = self.get_mod_path();
        self.get_one_mod(mods_root_path, name)
    }

    pub fn list_staging_folder(&self) -> Result<Vec<ModEntry>, ModManagerError> {
        let mods_staging_path =
            PathBuf::from(&self.config.staging_root_path).join(G::registry_id());
        self.list_folder(mods_staging_path)
    }

    pub fn get_staged_mod_by_name(&self, name: &str) -> Result<ModEntry, ModManagerError> {
        let staged_mods_path = self.get_staging_path();
        self.get_one_mod(staged_mods_path, name)
    }

    pub fn reconcile(&self) -> Result<Vec<ReconciledMod>, ModManagerError> {
        Ok(vec![])
    }

    fn list_folder(&self, source: PathBuf) -> Result<Vec<ModEntry>, ModManagerError> {
        let dir = match fs::read_dir(&source) {
            Ok(dir) => dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(ModManagerError::IoError(e)),
        };

        let mut entries = Vec::new();

        for entry in dir {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|e| ModManagerError::InvalidFilename(e.into_string().unwrap()))?;

            if entry.path().extension().is_some_and(|ext| ext == "zip") {
                entries.push(ModEntry {
                    name,
                    path: entry.path(),
                    kind: ModEntryKind::ZipArchive,
                });
            } else if entry.file_type()?.is_dir() {
                entries.push(ModEntry {
                    name,
                    path: entry.path(),
                    kind: ModEntryKind::Directory,
                });
            }
        }

        Ok(entries)
    }

    fn get_one_mod(&self, source: PathBuf, name: &str) -> Result<ModEntry, ModManagerError> {
        let dir = match fs::read_dir(&source) {
            Ok(dir) => dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(ModManagerError::ModNotFound(name.to_string()))
            }
            Err(e) => return Err(ModManagerError::IoError(e)),
        };

        for entry in dir {
            let entry = entry?;
            let entry_name = entry
                .file_name()
                .into_string()
                .map_err(|e| ModManagerError::InvalidFilename(e.into_string().unwrap()))?;

            if entry_name == name {
                if entry.path().extension().is_some_and(|ext| ext == "zip") {
                    return Ok(ModEntry {
                        name: entry_name,
                        path: entry.path(),
                        kind: ModEntryKind::ZipArchive,
                    });
                } else if entry.file_type()?.is_dir() {
                    return Ok(ModEntry {
                        name: entry_name,
                        path: entry.path(),
                        kind: ModEntryKind::Directory,
                    });
                }
            }
        }

        Err(ModManagerError::ModNotFound(name.to_string()))
    }

    fn get_mod_path(&self) -> PathBuf {
        PathBuf::from(&self.config.mods_root_path).join(G::registry_id())
    }

    fn get_staging_path(&self) -> PathBuf {
        PathBuf::from(&self.config.staging_root_path).join(G::registry_id())
    }
}

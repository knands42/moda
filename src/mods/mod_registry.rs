use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::installer::strip_zip_ext;
use crate::mods::mod_state::ModState;
use crate::mods::Installer;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ModMetadata {}
#[derive(Clone)]
pub struct ModEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: ModEntryKind,
    pub metadata: Option<ModMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModStatus {
    Downloaded,
    Staged,
    Enabled,
    Modified,
}

#[derive(Clone)]
pub struct ReconciledMod {
    pub name: String,
    pub status: ModStatus,
    pub source_entry: Option<ModEntry>,
    pub staging_entry: Option<ModEntry>,
    pub game_entry: Option<ModEntry>,
}

#[derive(Debug, Clone, PartialEq)]
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

    fn list_game_mods_folder(
        &self,
        game_mod_path: &Path,
    ) -> Result<Vec<ModEntry>, ModManagerError> {
        let dir = match fs::read_dir(game_mod_path) {
            Ok(dir) => dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(ModManagerError::IoError(e)),
        };

        let mut entries = Vec::new();
        for entry in dir {
            let entry = entry?;
            let ft = entry.file_type()?;
            if !ft.is_dir() && !ft.is_symlink() {
                continue;
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|e| ModManagerError::InvalidFilename(e.into_string().unwrap()))?;
            entries.push(ModEntry {
                name,
                path: entry.path(),
                kind: ModEntryKind::Directory,
                metadata: None,
            });
        }
        Ok(entries)
    }

    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        let source_mods = self.list_mods_folder()?;
        let staged_mods = self.list_staging_folder()?;
        let enabled_mods = self.list_game_mods_folder(game_mod_path)?;

        // Map effective name → source entry
        let mut source_by_name: std::collections::HashMap<String, ModEntry> =
            std::collections::HashMap::new();
        for m in &source_mods {
            let name = effective_name(m);
            source_by_name.insert(name, m.clone());
        }

        let mut names: Vec<String> = source_by_name.keys().cloned().collect();
        for m in &staged_mods {
            if !names.contains(&m.name) {
                names.push(m.name.clone());
            }
        }
        for m in &enabled_mods {
            if !names.contains(&m.name) {
                names.push(m.name.clone());
            }
        }

        let mut reconciled = Vec::new();
        for name in names {
            let src = source_by_name.get(&name).cloned();
            let stg = staged_mods.iter().find(|e| e.name == name).cloned();
            let ena = enabled_mods.iter().find(|e| e.name == name).cloned();

            let status = if let (Some(ref s), Some(ref t)) = (&src, &stg) {
                if is_newer(&s.path, &t.path) {
                    ModStatus::Modified
                } else if ena.is_some() {
                    ModStatus::Enabled
                } else {
                    ModStatus::Staged
                }
            } else if ena.is_some() {
                ModStatus::Enabled
            } else if stg.is_some() {
                ModStatus::Staged
            } else {
                ModStatus::Downloaded
            };

            reconciled.push(ReconciledMod {
                name,
                status,
                source_entry: src,
                staging_entry: stg,
                game_entry: ena,
            });
        }

        Ok(ModState { mods: reconciled })
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
                    metadata: None,
                });
            } else if entry.file_type()?.is_dir() {
                entries.push(ModEntry {
                    name,
                    path: entry.path(),
                    kind: ModEntryKind::Directory,
                    metadata: None,
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
                        metadata: None,
                    });
                } else if entry.file_type()?.is_dir() {
                    return Ok(ModEntry {
                        name: entry_name,
                        path: entry.path(),
                        kind: ModEntryKind::Directory,
                        metadata: None,
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

fn effective_name(entry: &ModEntry) -> String {
    if entry.kind == ModEntryKind::ZipArchive {
        match Installer::zip_wrap_directory(&entry.path) {
            Ok(Some(dir)) => dir,
            _ => strip_zip_ext(&entry.name),
        }
    } else {
        entry.name.clone()
    }
}

fn is_newer(a: &Path, b: &Path) -> bool {
    let a_mt = fs::metadata(a).ok().and_then(|m| m.modified().ok());
    let b_mt = fs::metadata(b).ok().and_then(|m| m.modified().ok());
    a_mt.zip(b_mt).is_some_and(|(a, b)| a > b)
}

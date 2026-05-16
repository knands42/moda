use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::Game;
use crate::mods::mod_state::ModState;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

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

fn strip_zip_ext(name: &str) -> String {
    name.strip_suffix(".zip").unwrap_or(name).to_string()
}

fn effective_name(entry: &ModEntry) -> String {
    if entry.kind == ModEntryKind::ZipArchive {
        zip_expected_name(&entry.path, &entry.name)
    } else {
        entry.name.clone()
    }
}

fn zip_expected_name(zip_path: &Path, zip_name: &str) -> String {
    match zip_top_level_dir(zip_path) {
        Ok(Some(dir)) => dir,
        _ => strip_zip_ext(zip_name),
    }
}

fn zip_top_level_dir(zip_path: &Path) -> Result<Option<String>, ModManagerError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| {
        ModManagerError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })?;

    let mut top_names: Vec<String> = Vec::new();
    let mut has_subdir = false;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| {
            ModManagerError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        let components: Vec<_> = path.components().collect();
        let Some(first) = components.first() else {
            continue;
        };
        let name = first.as_os_str().to_string_lossy().to_string();
        if !top_names.contains(&name) {
            top_names.push(name);
        }
        if components.len() > 1 {
            has_subdir = true;
        }
    }

    // Single unique top-level entry with subdirectory structure → wrapping directory
    if top_names.len() == 1 && has_subdir {
        Ok(Some(top_names[0].clone()))
    } else {
        Ok(None)
    }
}

fn is_newer(a: &Path, b: &Path) -> bool {
    let a_mt = fs::metadata(a).ok().and_then(|m| m.modified().ok());
    let b_mt = fs::metadata(b).ok().and_then(|m| m.modified().ok());
    a_mt.zip(b_mt).is_some_and(|(a, b)| a > b)
}

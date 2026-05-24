use crate::config::Config;
use crate::error::ModManagerError;
use crate::mods::installer::strip_zip_ext;
use crate::mods::mod_state::ModState;
use crate::mods::Installer;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ModMetadata {}
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

pub struct Catalog {
    config: Config,
    registry_id: &'static str,
}

impl Catalog {
    pub fn new(config: Config, registry_id: &'static str) -> Self {
        log::debug!("ModRegistry created for game: {}", registry_id);
        Self {
            config,
            registry_id,
        }
    }

    pub fn reconcile(&self, game_mod_path: &Path) -> Result<ModState, ModManagerError> {
        log::info!("Reconciling mods against {}", game_mod_path.display());
        let source_mods = self.list_mods_folder()?;
        let staged_mods = self.list_staging_folder()?;
        let enabled_mods = self.list_game_mods_folder(game_mod_path)?;

        // Map effective name → source entry
        let src_by_name: HashMap<String, ModEntry> = source_mods
            .iter()
            .map(|m| (effective_name(m), m.clone()))
            .collect();
        let stg_by_name: HashMap<String, ModEntry> = staged_mods
            .iter()
            .map(|m| (m.name.clone(), m.clone()))
            .collect();
        let ena_by_name: HashMap<String, ModEntry> = enabled_mods
            .iter()
            .map(|m| (m.name.clone(), m.clone()))
            .collect();

        let mut names: Vec<String> = src_by_name.keys().cloned().collect();
        for map in [&stg_by_name, &ena_by_name] {
            for name in map.keys() {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }

        let mut reconciled = HashMap::new();
        for name in names {
            let src = src_by_name.get(&name).cloned();
            let stg = stg_by_name.get(&name).cloned();
            let ena = ena_by_name.get(&name).cloned();

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

            reconciled.insert(
                name.clone(),
                ReconciledMod {
                    name,
                    status,
                    source_entry: src,
                    staging_entry: stg,
                    game_entry: ena,
                },
            );
        }

        Ok(ModState::new(reconciled))
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
            if !ft.is_symlink() {
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
        log::debug!("Found {} enabled mods in game folder", entries.len());
        Ok(entries)
    }

    fn list_mods_folder(&self) -> Result<Vec<ModEntry>, ModManagerError> {
        let mods_root_path = PathBuf::from(&self.config.mods_root_path).join(self.registry_id);
        let entries = self.list_folder(mods_root_path)?;
        log::debug!("Found {} mods in source folder", entries.len());
        Ok(entries)
    }

    fn list_staging_folder(&self) -> Result<Vec<ModEntry>, ModManagerError> {
        let mods_staging_path =
            PathBuf::from(&self.config.staging_root_path).join(self.registry_id);
        let entries = self.list_folder(mods_staging_path)?;
        log::debug!("Found {} mods in staging folder", entries.len());
        Ok(entries)
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

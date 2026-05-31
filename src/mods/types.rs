use std::path::PathBuf;

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
pub enum ModEntryKind {
    Directory,
    ZipArchive,
    RarArchive,
    Other,
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

pub fn allowed_extensions() -> &'static [&'static str] {
    static ALLOWED_EXTENSIONS: &[&str] = &["zip", "rar"];
    ALLOWED_EXTENSIONS
}

pub fn map_ext_to_kind(ext: &str) -> ModEntryKind {
    match ext {
        "zip" => ModEntryKind::ZipArchive,
        "rar" => ModEntryKind::RarArchive,
        _ => ModEntryKind::Other,
    }
}

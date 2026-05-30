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
    Other,
}

pub fn allowed_extensions() -> &'static [&'static str] {
    static ALLOWED_EXTENSIONS: &[&str] = &["zip", "rar"];
    ALLOWED_EXTENSIONS
}

pub fn map_ext_to_kind(ext: &str) -> ModEntryKind {
    match ext {
        "zip" => ModEntryKind::ZipArchive,
        "rar" => ModEntryKind::ZipArchive,
        _ => ModEntryKind::Other,
    }
}

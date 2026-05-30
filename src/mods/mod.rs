pub mod catalog;
pub mod downloader;
pub mod enabler;
mod installer;
mod mod_state;
mod orchestrator;

pub use enabler::SymlinkEnabler;
pub use installer::{strip_zip_ext, Installer, ModSource};
pub use mod_state::ModState;
pub use orchestrator::{SyncManager, SyncManagerOps};


#[derive(Debug, Clone, PartialEq)]
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
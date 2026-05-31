use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use moda::config::Config;
use moda::mods::repository::{ModRepository, TursoModRepository};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn make_config(tmp_dir: &TempDir) -> Config {
    let base_dir = tmp_dir.path().to_str().unwrap();
    Config {
        nexus_api_key: String::new(),
        mods_root_path: format!("{}/.moda/mods", base_dir),
        staging_root_path: format!("{}/.moda/staging", base_dir),
        game_search_paths: HashMap::new(),
        actual_config_path: PathBuf::from(base_dir)
            .join(".config")
            .join("moda")
            .join("config.toml"),
    }
}

pub fn create_zip(path: &Path, entries: &[&str]) {
    let f = fs::File::create(path).unwrap();
    let mut w = ZipWriter::new(f);
    for entry in entries {
        w.start_file(entry, SimpleFileOptions::default()).unwrap();
        w.write_all(b"content").unwrap();
    }
    w.finish().unwrap();
}

// --- Repository & ModState test helpers ---
pub fn new_repo() -> Arc<dyn ModRepository> {
    let tmp_dir = Box::new(TempDir::new().unwrap());
    let config = make_config(&tmp_dir);
    let repo = TursoModRepository::new(&config).unwrap();
    Box::leak(tmp_dir);
    repo
}

use moda::config::Config;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn make_config(mods_root: &str, staging_root: &str) -> Config {
    Config {
        nexus_api_key: String::new(),
        mods_root_path: mods_root.to_string(),
        staging_root_path: staging_root.to_string(),
        game_search_paths: HashMap::new(),
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

use moda::config::Config;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn make_config(tmp_dir: &TempDir) -> Config {
    std::env::set_var("HOME", tmp_dir.path().to_str().unwrap());
    let config = Config::new();

    config
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

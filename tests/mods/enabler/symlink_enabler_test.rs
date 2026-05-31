use moda::error::ModManagerError;
use moda::mods::enabler::Enabler;
use moda::mods::types::{ModEntry, ModEntryKind};
use moda::mods::SymlinkEnabler;
use std::fs;
use tempfile::TempDir;

fn make_entry(source: &std::path::Path, name: &str) -> ModEntry {
    ModEntry {
        name: name.into(),
        path: source.into(),
        kind: ModEntryKind::Directory,
        metadata: None,
    }
}

#[test]
fn test_enable_source_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let entry = make_entry(&temp_dir.path().join("nonexistent"), "target_dir");
    let target = temp_dir.path();

    let result = SymlinkEnabler::enable(&entry, target);

    assert!(result.is_err());
    match result {
        Err(ModManagerError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("Expected IoError with NotFound kind"),
    }
}

#[test]
fn test_enable_parent_directory_creates_intermediate_dirs() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let entry = make_entry(&source, "nested/dir/target");

    let result = SymlinkEnabler::enable(&entry, temp_dir.path()).unwrap();

    assert_eq!(result.name, "nested/dir/target");
    assert_eq!(result.path, temp_dir.path().join("nested/dir/target"));
    assert_eq!(result.kind, ModEntryKind::Directory);
    assert!(result.metadata.is_none());
    assert!(result.path.is_symlink());
    assert!(result.path.parent().unwrap().exists());
}

#[test]
fn test_enable_replaces_existing_symlink() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let old_target = temp_dir.path().join("old_target.txt");
    std::os::unix::fs::symlink(&source, &old_target).unwrap();
    assert!(old_target.is_symlink());

    let entry = make_entry(&source, "old_target");
    let game_mod_path = temp_dir.path();

    let result = SymlinkEnabler::enable(&entry, game_mod_path).unwrap();

    assert_eq!(result.name, "old_target");
    assert_eq!(result.path, game_mod_path.join("old_target"));
    assert!(result.path.is_symlink());
}

#[test]
fn test_enable_target_exists_not_symlink() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let collision = temp_dir.path().join("target.txt");
    fs::write(&collision, "existing file").unwrap();

    let entry = make_entry(&source, "target.txt");
    let result = SymlinkEnabler::enable(&entry, temp_dir.path());

    assert!(result.is_err());
    match result {
        Err(ModManagerError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
        }
        _ => panic!("Expected IoError with AlreadyExists kind"),
    }
}

#[test]
fn test_enable_creates_symlink_and_returns_modentry() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let entry = make_entry(&source, "SomeMod");
    let game_mods_path = temp_dir.path().join("Mods");

    let result = SymlinkEnabler::enable(&entry, &game_mods_path).unwrap();

    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.path, game_mods_path.join("SomeMod"));
    assert_eq!(result.kind, ModEntryKind::Directory);
    assert!(result.metadata.is_none());
    assert!(result.path.is_symlink());
    assert_eq!(fs::read_link(result.path).unwrap(), source);
}

#[test]
fn test_disable_removes_symlink() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    let target = temp_dir.path().join("target");
    fs::write(&source, "content").unwrap();
    SymlinkEnabler::activate(&source, &target).unwrap();
    assert!(target.is_symlink());

    let entry = make_entry(&target, "target");

    SymlinkEnabler::disable(&entry).unwrap();

    assert!(!target.exists());
}

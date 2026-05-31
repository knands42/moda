use moda::mods::stager::{DirectCopyStager, Stager};
use moda::mods::types::{ModEntry, ModEntryKind};
use tempfile::tempdir;

#[test]
fn test_stage_copies_dir_and_returns_entry() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("SomeMod");
    let nested_dir = source_dir.join("subfolder");
    let nested_level_2_dir = nested_dir.join("subsubfolder");
    std::fs::create_dir_all(&nested_level_2_dir).unwrap();
    std::fs::write(source_dir.join("mod1.txt"), "mod1 info").unwrap();
    std::fs::write(nested_dir.join("mod2.txt"), "mod2 info").unwrap();
    std::fs::write(nested_level_2_dir.join("mod3.txt"), "mod3 info").unwrap();

    let entry = ModEntry {
        name: "SomeMod".into(),
        path: source_dir,
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let staging_path = temp.path().join("staging");

    let result = DirectCopyStager::stage(&entry, &staging_path).unwrap();

    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.path, staging_path.join("SomeMod"));
    assert_eq!(result.kind, ModEntryKind::Directory);
    assert!(result.metadata.is_none());

    let target = &result.path;
    assert!(target.join("mod1.txt").exists());
    assert!(target.join("subfolder").is_dir());
    assert!(target.join("subfolder/mod2.txt").exists());
    assert!(target.join("subfolder/subsubfolder").is_dir());
    assert!(target.join("subfolder/subsubfolder/mod3.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target.join("mod1.txt")).unwrap(),
        "mod1 info"
    );
}

#[test]
fn test_uninstall_removes_directory() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("my_mod");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("file.txt"), "data").unwrap();

    DirectCopyStager::unstage(&dir).unwrap();

    assert!(!dir.exists());
}

#[test]
fn test_uninstall_not_found_does_not_error() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("nonexistent_mod");

    let result = DirectCopyStager::unstage(&dir);

    assert!(result.is_ok());
}

#[test]
fn test_get_mod_name_folder_returns_name() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("anything");

    let result = DirectCopyStager::get_mod_name(&path).unwrap();

    assert_eq!(result, "anything");
}

#[test]
fn test_get_mod_name_multiple_path_components() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("mods").join("anything");

    let result = DirectCopyStager::get_mod_name(&path).unwrap();

    assert_eq!(result, "anything");
}

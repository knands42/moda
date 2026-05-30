use moda::mods::{DirectCopyInstaller, Installer};
use tempfile::tempdir;

#[test]
fn test_install_from_dir() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let nested_dir = source_dir.join("subfolder");
    let nested_level_2_dir = nested_dir.join("subsubfolder");
    std::fs::create_dir(&source_dir).unwrap();
    std::fs::create_dir(&nested_dir).unwrap();
    std::fs::create_dir(&nested_level_2_dir).unwrap();

    let source_file = source_dir.join("mod1.txt");
    let nested_file = nested_dir.join("mod2.txt");
    let nested_level_2_file = nested_level_2_dir.join("mod3.txt");
    std::fs::write(&source_file, "mod1 info").unwrap();
    std::fs::write(&nested_file, "mod2 info").unwrap();
    std::fs::write(&nested_level_2_file, "mod3 info").unwrap();

    let target_dir = temp.path().join("target");
    std::fs::create_dir(&target_dir).unwrap();

    DirectCopyInstaller::install(&source_dir, &target_dir).unwrap();

    assert!(target_dir.join("mod1.txt").exists());
    assert!(target_dir.join("subfolder").is_dir());
    assert!(target_dir.join("subfolder/mod2.txt").exists());
    assert!(target_dir.join("subfolder/subsubfolder").is_dir());
    assert!(target_dir.join("subfolder/subsubfolder/mod3.txt").exists());

    assert_eq!(
        std::fs::read_to_string(target_dir.join("mod1.txt")).unwrap(),
        "mod1 info"
    );
    assert_eq!(
        std::fs::read_to_string(target_dir.join("subfolder/mod2.txt")).unwrap(),
        "mod2 info"
    );
    assert_eq!(
        std::fs::read_to_string(target_dir.join("subfolder/subsubfolder/mod3.txt")).unwrap(),
        "mod3 info"
    )
}

#[test]
fn test_uninstall_removes_directory() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("my_mod");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("file.txt"), "data").unwrap();

    DirectCopyInstaller::uninstall(&dir).unwrap();

    assert!(!dir.exists());
}

#[test]
fn test_uninstall_not_found_does_not_error() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("nonexistent_mod");

    let result = DirectCopyInstaller::uninstall(&dir);

    assert!(result.is_ok());
}

#[test]
fn test_get_mod_name_folder_returns_name() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("anything");

    let result = DirectCopyInstaller::get_mod_name_from_installer(&path).unwrap();

    assert_eq!(result, "anything");
}

#[test]
fn test_get_mod_name_multiple_path_components() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("/mods/anything");

    let result = DirectCopyInstaller::get_mod_name_from_installer(&path).unwrap();

    assert_eq!(result, "anything");
}

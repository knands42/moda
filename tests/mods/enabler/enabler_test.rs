use moda::mods::enabler::Enabler;
use moda::error::ModManagerError;
use moda::mods::SymlinkEnabler;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_activate_source_not_exists() {
    // Given: a source path that does not exist
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("nonexistent");
    let target = temp_dir.path().join("target");

    // When: activate is called with the non-existent source
    let result = SymlinkEnabler::activate(&source, &target);

    // Then: it returns a NotFound IoError
    assert!(result.is_err());
    match result {
        Err(ModManagerError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("Expected IoError with NotFound kind"),
    }
}

#[test]
fn test_activate_parent_directory_created() {
    // Given: a source file and a deeply nested target path
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let target = temp_dir.path().join("nested/dir/target.txt");

    // When: activate is called
    let result = SymlinkEnabler::activate(&source, &target);

    // Then: a symlink is created and the parent directory exists
    assert!(result.is_ok());
    assert!(target.exists());
    assert!(target.is_symlink());
    assert!(target.parent().unwrap().exists());
}

#[test]
fn test_activate_existing_symlink_removed() {
    // Given: an existing symlink at the target path
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let target = temp_dir.path().join("target.txt");
    std::os::unix::fs::symlink(&source, &target).unwrap();
    assert!(target.is_symlink());

    // When: activate is called again for the same target
    let result = SymlinkEnabler::activate(&source, &target);

    // Then: the old symlink is replaced and a new one is created
    assert!(result.is_ok());
    assert!(target.is_symlink());
}

#[test]
fn test_activate_target_exists_not_symlink() {
    // Given: a real file at the target path (not a symlink)
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let target = temp_dir.path().join("target.txt");
    fs::write(&target, "existing file").unwrap();

    // When: activate is called
    let result = SymlinkEnabler::activate(&source, &target);

    // Then: it returns an AlreadyExists IoError
    assert!(result.is_err());
    match result {
        Err(ModManagerError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
        }
        _ => panic!("Expected IoError with AlreadyExists kind"),
    }
}

#[test]
fn test_activate_successful_symlink() {
    // Given: a source file and a target path
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();
    let target = temp_dir.path().join("target.txt");

    // When: activate is called
    let result = SymlinkEnabler::activate(&source, &target);

    // Then: a symlink is created successfully
    assert!(result.is_ok());
    assert!(target.exists());
    assert!(target.is_symlink());
}

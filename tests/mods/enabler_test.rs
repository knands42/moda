use moda::error::ModManagerError;
use moda::mods::Enabler;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_activate_source_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("nonexistent");
    let target = temp_dir.path().join("target");

    let result = Enabler::activate(&source, &target);

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
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let target = temp_dir.path().join("nested/dir/target.txt");

    let result = Enabler::activate(&source, &target);

    assert!(result.is_ok());
    assert!(target.exists());
    assert!(target.is_symlink());
    assert!(target.parent().unwrap().exists());
}

#[test]
fn test_activate_existing_symlink_removed() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let target = temp_dir.path().join("target.txt");
    std::os::unix::fs::symlink(&source, &target).unwrap();

    // Verify symlink exists before
    assert!(target.is_symlink());

    let result = Enabler::activate(&source, &target);

    assert!(result.is_ok());
    assert!(target.is_symlink());
}

#[test]
fn test_activate_target_exists_not_symlink() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let target = temp_dir.path().join("target.txt");
    fs::write(&target, "existing file").unwrap();

    let result = Enabler::activate(&source, &target);

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
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    fs::write(&source, "test content").unwrap();

    let target = temp_dir.path().join("target.txt");

    let result = Enabler::activate(&source, &target);

    assert!(result.is_ok());
    assert!(target.exists());
    assert!(target.is_symlink());
}

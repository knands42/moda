use moda::mods::{strip_zip_ext, Installer, ZipInstaller};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_install_from_zip() {
    // Given: a zip file with a file at root and a file in a subfolder
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let target_dir = temp.path().join("target");
    std::fs::create_dir(&source_dir).unwrap();
    std::fs::create_dir(&target_dir).unwrap();

    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.write_all(b"mod info").unwrap();
    zip_writer
        .start_file("subfolder/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.write_all(b"mod info inside folder").unwrap();
    zip_writer.finish().unwrap();

    // When: installing from the zip
    ZipInstaller::install(&zip_path, &target_dir).unwrap();

    // Then: files are extracted to the correct locations
    assert!(target_dir.join("mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target_dir.join("mod.txt")).unwrap(),
        "mod info"
    );
    assert!(target_dir.join("subfolder").is_dir());
    assert!(target_dir.join("subfolder/mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target_dir.join("subfolder/mod.txt")).unwrap(),
        "mod info inside folder"
    );
}

#[test]
fn test_strip_zip_ext_removes_zip_extension() {
    // Given: filenames ending in .zip
    // When: strip_zip_ext is called
    // Then: the .zip extension is removed
    assert_eq!(strip_zip_ext("SomeMod.zip"), "SomeMod");
    assert_eq!(strip_zip_ext("archive.zip"), "archive");
}

#[test]
fn test_strip_zip_ext_no_extension() {
    // Given: filenames without .zip or with other extensions
    // When: strip_zip_ext is called
    // Then: the filename is returned unchanged
    assert_eq!(strip_zip_ext("SomeMod"), "SomeMod");
    assert_eq!(strip_zip_ext("SomeMod.tar.gz"), "SomeMod.tar.gz");
}

#[test]
fn test_zip_wrap_directory_single_wrapping_dir() {
    // Given: a zip with all files inside a single top-level directory
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("SomeMod/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file("SomeMod/fonts/other.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path).unwrap();

    // Then: it returns the wrapping directory name
    assert_eq!(result, "SomeMod");
}

#[test]
fn test_zip_wrap_directory_files_at_root() {
    // Given: a zip with multiple files at root level
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file("readme.md", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path);

    // Then: it returns an error (no single wrapping directory)
    assert!(result.is_err());
}

#[test]
fn test_zip_wrap_directory_single_file_at_root() {
    // Given: a zip with a single file at root level
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path);

    // Then: it returns an error (single file at root is not a wrap dir)
    assert!(result.is_err());
}

#[test]
fn test_zip_wrap_directory_multiple_top_level_dirs() {
    // Given: a zip with multiple top-level directories
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("folder1/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file("folder2/other.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path);

    // Then: it returns an error (multiple top-level dirs)
    assert!(result.is_err());
}

#[test]
fn test_zip_wrap_directory_nested_single_dir() {
    // Given: a zip with deeply nested files under a single top-level directory
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("mod.zip");
    let zip_file = File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("SomeMod-1.0.0/sub/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file(
            "SomeMod-1.0.0/sub/deep/asset.dat",
            SimpleFileOptions::default(),
        )
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path).unwrap();

    // Then: it returns the top-level wrapping directory name
    assert_eq!(result, "SomeMod-1.0.0");
}

#[test]
fn test_zip_wrap_directory_invalid_path() {
    // Given: a path to a non-existent zip file
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("nonexistent.zip");

    // When: checking for wrap directory
    let result = ZipInstaller::get_mod_name_from_installer(&zip_path);

    // Then: it returns an error
    assert!(result.is_err());
}

use moda::mods::{strip_zip_ext, Installer, ModSource};
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
    Installer::install(&ModSource::LocalZip(zip_path), &target_dir).unwrap();

    // Then: files are extracted to the correct locations
    assert!(target_dir.join("mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target_dir.join("mod.txt")).unwrap(),
        "mod info"
    );
    assert!(target_dir.join("subfolder").is_dir());
    assert!(target_dir.join("subfolder/mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(&target_dir.join("subfolder/mod.txt")).unwrap(),
        "mod info inside folder"
    );
}

#[test]
fn test_install_from_dir() {
    // Given: a source directory with nested files
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

    // When: installing from the directory
    Installer::install(&ModSource::LocalDir(source_dir), &target_dir).unwrap();

    // Then: all files and directories are recursively copied
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
        .start_file("SomeMod/assets/other.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    // When: checking for wrap directory
    let result = Installer::zip_wrap_directory(&zip_path).unwrap();

    // Then: it returns the wrapping directory name
    assert_eq!(result, Some("SomeMod".to_string()));
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
    let result = Installer::zip_wrap_directory(&zip_path).unwrap();

    // Then: it returns None (no single wrapping directory)
    assert_eq!(result, None);
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
    let result = Installer::zip_wrap_directory(&zip_path).unwrap();

    // Then: it returns None (single file at root is not a wrap dir)
    assert_eq!(result, None);
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
    let result = Installer::zip_wrap_directory(&zip_path).unwrap();

    // Then: it returns None (multiple top-level dirs)
    assert_eq!(result, None);
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
    let result = Installer::zip_wrap_directory(&zip_path).unwrap();

    // Then: it returns the top-level wrapping directory name
    assert_eq!(result, Some("SomeMod-1.0.0".to_string()));
}

#[test]
fn test_zip_wrap_directory_invalid_path() {
    // Given: a path to a non-existent zip file
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("nonexistent.zip");

    // When: checking for wrap directory
    let result = Installer::zip_wrap_directory(&zip_path);

    // Then: it returns an error
    assert!(result.is_err());
}

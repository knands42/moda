use moda::mods::stager::{strip_zip_ext, Stager, ZipStager};
use moda::mods::types::{ModEntry, ModEntryKind};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_stage_flat_zip_extracts_and_returns_entry() {
    let temp = tempdir().unwrap();
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

    let entry = ModEntry {
        name: "mod.zip".into(),
        path: zip_path,
        kind: ModEntryKind::ZipArchive,
        metadata: None,
    };
    let staging_path = temp.path().join("staging");

    let result = ZipStager::stage(&entry, &staging_path).unwrap();

    assert_eq!(result.name, "mod");
    assert_eq!(result.path, staging_path.join("mod"));
    assert_eq!(result.kind, ModEntryKind::Directory);
    assert!(result.metadata.is_none());

    let target = &result.path;
    assert!(target.join("mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target.join("mod.txt")).unwrap(),
        "mod info"
    );
    assert!(target.join("subfolder").is_dir());
    assert!(target.join("subfolder/mod.txt").exists());
    assert_eq!(
        std::fs::read_to_string(target.join("subfolder/mod.txt")).unwrap(),
        "mod info inside folder"
    );
}

#[test]
fn test_zip_wrap_directory() {
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

    let entry = ModEntry {
        name: "mod.zip".into(),
        path: zip_path,
        kind: ModEntryKind::ZipArchive,
        metadata: None,
    };
    let staging_path = temp.path().join("staging");

    let result = ZipStager::stage(&entry, &staging_path).unwrap();

    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.path, staging_path.join("SomeMod"));
    assert_eq!(result.kind, ModEntryKind::Directory);
    assert!(result.metadata.is_none());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    assert!(staging_path.join("SomeMod").join("fonts").is_dir());
    assert!(staging_path.join("SomeMod/fonts/other.txt").exists());
}

#[test]
fn test_zip_directory_multiple_top_level_dirs() {
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

    let result = ZipStager::get_mod_name(&zip_path);

    assert!(result.is_err());
}

#[test]
fn test_strip_zip_ext_removes_zip_extension() {
    assert_eq!(strip_zip_ext("SomeMod.zip"), "SomeMod");
    assert_eq!(strip_zip_ext("archive.zip"), "archive");
}

#[test]
fn test_strip_zip_ext_no_extension() {
    assert_eq!(strip_zip_ext("SomeMod"), "SomeMod");
    assert_eq!(strip_zip_ext("SomeMod.tar.gz"), "SomeMod.tar.gz");
}

#[test]
fn test_zip_wrap_directory_invalid_path() {
    let temp = tempdir().unwrap();
    let zip_path = temp.path().join("nonexistent.zip");

    let result = ZipStager::get_mod_name(&zip_path);

    assert!(result.is_err());
}

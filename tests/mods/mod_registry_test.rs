use moda::config::Config;
use moda::error::ModManagerError;
use moda::games::StardewValley;
use moda::mods::mod_registry::{ModEntryKind, ModRegistry, ModStatus};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn make_config(mods_root: &str, staging_root: &str) -> Config {
    Config {
        nexus_api_key: String::new(),
        mods_root_path: mods_root.to_string(),
        staging_root_path: staging_root.to_string(),
        game_search_paths: HashMap::new(),
    }
}

#[test]
fn test_list_mods_folder_empty_when_not_exists() {
    // Given: a mods folder that does not exist
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the mods folder
    let result = registry.list_mods_folder().unwrap();

    // Then: an empty list is returned
    assert!(result.is_empty());
}

#[test]
fn test_list_mods_folder_returns_dirs() {
    // Given: a mods folder with two directory mods
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir(mods_path.join("AnotherMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the mods folder
    let mut result = registry.list_mods_folder().unwrap();
    result.sort_by(|a, b| a.name.cmp(&b.name));

    // Then: both directories are returned with Directory kind
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "AnotherMod");
    assert_eq!(result[0].kind, ModEntryKind::Directory);
    assert_eq!(result[1].name, "SomeMod");
    assert_eq!(result[1].kind, ModEntryKind::Directory);
}

#[test]
fn test_list_mods_folder_returns_zips() {
    // Given: a mods folder with a zip file
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::write(mods_path.join("SomeMod.zip"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the mods folder
    let result = registry.list_mods_folder().unwrap();

    // Then: the zip file is returned with ZipArchive kind
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "SomeMod.zip");
    assert_eq!(result[0].kind, ModEntryKind::ZipArchive);
}

#[test]
fn test_list_mods_folder_skips_other_files() {
    // Given: a mods folder with only non-mod files (txt, png)
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::write(mods_path.join("readme.txt"), "ignore").unwrap();
    fs::write(mods_path.join("image.png"), "ignore").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the mods folder
    let result = registry.list_mods_folder().unwrap();

    // Then: the list is empty (non-mod files are skipped)
    assert!(result.is_empty());
}

#[test]
fn test_get_mod_by_name_found_dir() {
    // Given: a mods folder with a directory mod
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: looking up the mod by name
    let result = registry.get_mod_by_name("SomeMod").unwrap();

    // Then: the correct directory entry is returned
    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.kind, ModEntryKind::Directory);
}

#[test]
fn test_get_mod_by_name_found_zip() {
    // Given: a mods folder with a zip mod
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::write(mods_path.join("SomeMod.zip"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: looking up the mod by its zip name
    let result = registry.get_mod_by_name("SomeMod.zip").unwrap();

    // Then: the correct zip entry is returned
    assert_eq!(result.name, "SomeMod.zip");
    assert_eq!(result.kind, ModEntryKind::ZipArchive);
}

#[test]
fn test_get_mod_by_name_not_found() {
    // Given: a mods folder with a mod that does not match the requested name
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: looking up a non-existent mod
    let result = registry.get_mod_by_name("NonExistent");

    // Then: a ModNotFound error is returned
    assert!(result.is_err());
    match result {
        Err(ModManagerError::ModNotFound(name)) => assert_eq!(name, "NonExistent"),
        _ => panic!("Expected ModNotFound"),
    }
}

#[test]
fn test_get_mod_by_name_folder_not_exists() {
    // Given: no mods folder exists at all
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: looking up any mod
    let result = registry.get_mod_by_name("AnyMod");

    // Then: a ModNotFound error is returned
    assert!(result.is_err());
    match result {
        Err(ModManagerError::ModNotFound(name)) => assert_eq!(name, "AnyMod"),
        _ => panic!("Expected ModNotFound"),
    }
}

#[test]
fn test_list_game_mods_folder_not_exists() {
    // Given: a game mods folder that does not exist
    let temp = TempDir::new().unwrap();
    let game_mod_path = temp.path().join("game").join("Mods");

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the game mods folder
    let result = registry.list_game_mods_folder(&game_mod_path).unwrap();

    // Then: an empty list is returned
    assert!(result.is_empty());
}

#[test]
fn test_list_game_mods_folder_returns_symlinks() {
    // Given: a game mods folder with a symlink to a staged mod
    let temp = TempDir::new().unwrap();
    let game_mod_path = temp.path().join("game").join("Mods");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir_all(&game_mod_path).unwrap();
    std::os::unix::fs::symlink(staging_path.join("SomeMod"), game_mod_path.join("SomeMod"))
        .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the game mods folder
    let result = registry.list_game_mods_folder(&game_mod_path).unwrap();

    // Then: the symlink is returned as a Directory entry
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "SomeMod");
    assert_eq!(result[0].kind, ModEntryKind::Directory);
}

#[test]
fn test_list_game_mods_folder_skips_non_symlinks() {
    // Given: a game mods folder with a real directory, a real file, and a symlink
    let temp = TempDir::new().unwrap();
    let game_mod_path = temp.path().join("game").join("Mods");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir_all(&game_mod_path).unwrap();
    fs::create_dir(game_mod_path.join("RealDir")).unwrap();
    fs::write(game_mod_path.join("file.txt"), "content").unwrap();
    std::os::unix::fs::symlink(staging_path.join("SomeMod"), game_mod_path.join("SomeMod"))
        .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: listing the game mods folder
    let result = registry.list_game_mods_folder(&game_mod_path).unwrap();

    // Then: only the symlink is returned; real dirs and files are skipped
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "SomeMod");
}

#[test]
fn test_reconcile_empty() {
    // Given: no mods in any folder (source, staging, or game)
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the reconciled state is empty
    assert!(result.snapshot().is_empty());
}

#[test]
fn test_reconcile_enabled_failed_if_not_symlink_mod() {
    // Given: a mod in source and staging, and a real directory (not a symlink) in game mods
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir(game_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod is Staged (real dir is not a symlink, so not detected as enabled)
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Staged);
    assert!(result.snapshot()[0].source_entry.is_some());
    assert!(result.snapshot()[0].staging_entry.is_some());
    assert!(result.snapshot()[0].game_entry.is_none());
}

#[test]
fn test_reconcile_zip_mod() {
    // Given: a zip archive in the source folder (no staging or game entry)
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    let zip_path = mods_path.join("SomeMod.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.write_all(b"content").unwrap();
    zip_writer.finish().unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod is Downloaded with a zip source entry
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Downloaded);
    assert!(result.snapshot()[0].source_entry.is_some());
    assert_eq!(
        result.snapshot()[0].source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
}

#[test]
fn test_reconcile_wrapped_zip_matches_staging() {
    // Given: a zip that wraps in a directory matching the staging entry name
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();

    let zip_path = mods_path.join("SomeMod.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("SomeMod/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling (zip wraps SomeMod, effective name matches, but no staging dir yet)
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod appears as Downloaded with effective name matching the wrap dir
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_reconcile_wrapped_zip_different_name() {
    // Given: a zip named "Mod.zip" that wraps as "SomeMod-v2/"
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();

    let zip_path = mods_path.join("Mod.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("SomeMod-v2/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file("SomeMod-v2/sub/asset.dat", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling (effective name is the wrap dir, not the zip name)
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the effective name is the wrapping directory name
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod-v2");
    assert_eq!(result.snapshot()[0].status, ModStatus::Downloaded);
    assert_eq!(
        result.snapshot()[0].source_entry.as_ref().unwrap().name,
        "Mod.zip"
    );
}

#[test]
fn test_reconcile_zip_with_dir_and_root_files() {
    // Given: a zip with both a wrapping dir and a file at root (mixed structure)
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();

    let zip_path = mods_path.join("SomeMod-1.0.0.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("SomeMod/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer
        .start_file("readme.md", SimpleFileOptions::default())
        .unwrap();
    zip_writer.finish().unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling (no single wrap dir due to mixed root)
    let result = registry.reconcile(&game_path).unwrap();

    // Then: fallback to strip_zip_ext for the effective name
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod-1.0.0");
    assert_eq!(result.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_reconcile_multiple_mixed_states() {
    // Given: three mods — one Downloaded, one Staged, one Enabled
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();

    fs::create_dir(mods_path.join("ModA")).unwrap();

    fs::create_dir(mods_path.join("ModB")).unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();

    fs::create_dir(mods_path.join("ModC")).unwrap();
    fs::create_dir(staging_path.join("ModC")).unwrap();
    std::os::unix::fs::symlink(staging_path.join("ModC"), game_path.join("ModC")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: each mod has the correct status
    assert_eq!(result.snapshot().len(), 3);

    let snapshot = result.snapshot();
    let mod_a = snapshot.iter().find(|m| m.name == "ModA").unwrap();
    assert_eq!(mod_a.status, ModStatus::Downloaded);

    let snapshot = result.snapshot();
    let mod_b = snapshot.iter().find(|m| m.name == "ModB").unwrap();
    assert_eq!(mod_b.status, ModStatus::Staged);

    let snapshot = result.snapshot();
    let mod_c = snapshot.iter().find(|m| m.name == "ModC").unwrap();
    assert_eq!(mod_c.status, ModStatus::Enabled);
}

#[test]
fn test_reconcile_enabled_mod_without_staging() {
    // Given: a mod with a source entry and a game symlink, but no staging entry
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    std::os::unix::fs::symlink(mods_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod is Enabled (source + game, no staging)
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Enabled);
    assert!(result.snapshot()[0].source_entry.is_some());
    assert!(result.snapshot()[0].staging_entry.is_none());
    assert!(result.snapshot()[0].game_entry.is_some());
}

#[test]
fn test_reconcile_orphan_enabled() {
    // Given: a symlink in the game mods folder with no corresponding source or staging
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let orphan_target = temp.path().join("orphan_target");
    fs::create_dir(&orphan_target).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    std::os::unix::fs::symlink(&orphan_target, game_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the orphan symlink is still detected as Enabled with only a game entry
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Enabled);
    assert!(result.snapshot()[0].source_entry.is_none());
    assert!(result.snapshot()[0].staging_entry.is_none());
    assert!(result.snapshot()[0].game_entry.is_some());
}

#[test]
fn test_reconcile_orphan_staged() {
    // Given: a staging entry with no corresponding source or game entry
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the orphan staging entry is detected as Staged
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Staged);
    assert!(result.snapshot()[0].source_entry.is_none());
    assert!(result.snapshot()[0].staging_entry.is_some());
    assert!(result.snapshot()[0].game_entry.is_none());
}

#[test]
fn test_reconcile_modified_with_enabled() {
    // Given: a staged mod with a newer source AND an active game symlink
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&game_path).unwrap();

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("file.txt"), "stale").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::write(mods_path.join("SomeMod").join("file.txt"), "newer").unwrap();

    std::os::unix::fs::symlink(staging_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod is Modified (newer source takes priority over enabled status)
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Modified);
    assert!(result.snapshot()[0].source_entry.is_some());
    assert!(result.snapshot()[0].staging_entry.is_some());
    assert!(result.snapshot()[0].game_entry.is_some());
}

#[test]
fn test_reconcile_enabled_with_staging_only_no_source() {
    // Given: a mod only in staging and game (source was removed)
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    std::os::unix::fs::symlink(staging_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    // When: reconciling all folders
    let result = registry.reconcile(&game_path).unwrap();

    // Then: the mod is still Enabled (staging + game, no source needed)
    assert_eq!(result.snapshot().len(), 1);
    assert_eq!(result.snapshot()[0].name, "SomeMod");
    assert_eq!(result.snapshot()[0].status, ModStatus::Enabled);
    assert!(result.snapshot()[0].source_entry.is_none());
    assert!(result.snapshot()[0].staging_entry.is_some());
    assert!(result.snapshot()[0].game_entry.is_some());
}

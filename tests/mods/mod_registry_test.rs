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
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.list_mods_folder().unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_list_mods_folder_returns_dirs() {
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

    let mut result = registry.list_mods_folder().unwrap();
    result.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "AnotherMod");
    assert_eq!(result[0].kind, ModEntryKind::Directory);
    assert_eq!(result[1].name, "SomeMod");
    assert_eq!(result[1].kind, ModEntryKind::Directory);
}

#[test]
fn test_list_mods_folder_returns_zips() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::write(mods_path.join("SomeMod.zip"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.list_mods_folder().unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "SomeMod.zip");
    assert_eq!(result[0].kind, ModEntryKind::ZipArchive);
}

#[test]
fn test_list_mods_folder_skips_other_files() {
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

    let result = registry.list_mods_folder().unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_get_mod_by_name_found_dir() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.get_mod_by_name("SomeMod").unwrap();

    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.kind, ModEntryKind::Directory);
}

#[test]
fn test_get_mod_by_name_found_zip() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::write(mods_path.join("SomeMod.zip"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.get_mod_by_name("SomeMod.zip").unwrap();

    assert_eq!(result.name, "SomeMod.zip");
    assert_eq!(result.kind, ModEntryKind::ZipArchive);
}

#[test]
fn test_get_mod_by_name_not_found() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.get_mod_by_name("NonExistent");

    assert!(result.is_err());
    match result {
        Err(ModManagerError::ModNotFound(name)) => assert_eq!(name, "NonExistent"),
        _ => panic!("Expected ModNotFound"),
    }
}

#[test]
fn test_get_mod_by_name_folder_not_exists() {
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.get_mod_by_name("AnyMod");

    assert!(result.is_err());
    match result {
        Err(ModManagerError::ModNotFound(name)) => assert_eq!(name, "AnyMod"),
        _ => panic!("Expected ModNotFound"),
    }
}

#[test]
fn test_list_staging_folder_empty_when_not_exists() {
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.list_staging_folder().unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_get_staged_mod_by_name_found() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.get_staged_mod_by_name("SomeMod").unwrap();

    assert_eq!(result.name, "SomeMod");
    assert_eq!(result.kind, ModEntryKind::Directory);
}

#[test]
fn test_reconcile_empty() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.reconcile(&game_path).unwrap();

    assert!(result.mods.is_empty());
}

#[test]
fn test_reconcile_downloaded_mod() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 1);
    assert_eq!(result.mods[0].name, "SomeMod");
    assert_eq!(result.mods[0].status, ModStatus::Downloaded);
    assert!(result.mods[0].source_entry.is_some());
    assert!(result.mods[0].staging_entry.is_none());
    assert!(result.mods[0].game_entry.is_none());
}

#[test]
fn test_reconcile_staged_mod() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 1);
    assert_eq!(result.mods[0].name, "SomeMod");
    assert_eq!(result.mods[0].status, ModStatus::Staged);
    assert!(result.mods[0].source_entry.is_some());
    assert!(result.mods[0].staging_entry.is_some());
    assert!(result.mods[0].game_entry.is_none());
}

#[test]
fn test_reconcile_enabled_mod() {
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

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 1);
    assert_eq!(result.mods[0].name, "SomeMod");
    assert_eq!(result.mods[0].status, ModStatus::Enabled);
    assert!(result.mods[0].source_entry.is_some());
    assert!(result.mods[0].staging_entry.is_some());
    assert!(result.mods[0].game_entry.is_some());
}

#[test]
fn test_reconcile_modified_mod() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    // Create staging dir first so source dir will have a newer mtime
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("file.txt"), "stale").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    // Source dir created after — its mtime is newer
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::write(mods_path.join("SomeMod").join("file.txt"), "newer").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 1);
    assert_eq!(result.mods[0].name, "SomeMod");
    assert_eq!(result.mods[0].status, ModStatus::Modified);
}

#[test]
fn test_reconcile_zip_mod() {
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

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 1);
    assert_eq!(result.mods[0].name, "SomeMod");
    assert_eq!(result.mods[0].status, ModStatus::Downloaded);
    assert!(result.mods[0].source_entry.is_some());
    assert_eq!(
        result.mods[0].source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
}

#[test]
fn test_reconcile_multiple_mixed_states() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();

    // Downloaded mod
    fs::create_dir(mods_path.join("ModA")).unwrap();

    // Staged mod
    fs::create_dir(mods_path.join("ModB")).unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();

    // Enabled mod
    fs::create_dir(mods_path.join("ModC")).unwrap();
    fs::create_dir(staging_path.join("ModC")).unwrap();
    fs::create_dir(game_path.join("ModC")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let registry: ModRegistry<StardewValley> = ModRegistry::new(config);

    let result = registry.reconcile(&game_path).unwrap();

    assert_eq!(result.mods.len(), 3);

    let mod_a = result.mods.iter().find(|m| m.name == "ModA").unwrap();
    assert_eq!(mod_a.status, ModStatus::Downloaded);

    let mod_b = result.mods.iter().find(|m| m.name == "ModB").unwrap();
    assert_eq!(mod_b.status, ModStatus::Staged);

    let mod_c = result.mods.iter().find(|m| m.name == "ModC").unwrap();
    assert_eq!(mod_c.status, ModStatus::Enabled);
}

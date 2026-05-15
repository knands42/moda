use moda::config::Config;
use moda::error::ModManagerError;
use moda::games::StardewValley;
use moda::mods::mod_registry::{ModEntryKind, ModRegistry};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

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

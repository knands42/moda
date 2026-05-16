use moda::config::Config;
use moda::games::StardewValley;
use moda::mods::mod_registry::{ModEntry, ModEntryKind, ModStatus, ReconciledMod};
use moda::mods::ModState;
use moda::mods::SyncManager;
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
fn test_stage_mods_empty_folder() {
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    let result = manager.stage_mods(&mut state);

    assert!(result.is_ok());
}

#[test]
fn test_stage_one_mod_dir() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::write(mods_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: mods_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: mods_path.join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.stage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_stage_one_mod_zip() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
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
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod.zip".to_string(),
            path: zip_path.clone(),
            kind: ModEntryKind::ZipArchive,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod.zip".to_string(),
        path: zip_path,
        kind: ModEntryKind::ZipArchive,
        metadata: None,
    };
    let result = manager.stage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_stage_one_mod_zip_with_wrap_directory() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    let zip_path = mods_path.join("SomeMod.zip");
    let zip_file = fs::File::create(&zip_path).unwrap();
    let mut zip_writer = ZipWriter::new(zip_file);
    zip_writer
        .start_file("WrapDir/mod.txt", SimpleFileOptions::default())
        .unwrap();
    zip_writer.write_all(b"content").unwrap();
    zip_writer.finish().unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod.zip".to_string(),
            path: zip_path.clone(),
            kind: ModEntryKind::ZipArchive,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod.zip".to_string(),
        path: zip_path,
        kind: ModEntryKind::ZipArchive,
        metadata: None,
    };
    let result = manager.stage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    // Wrapping zip extracts into staging root, creating WrapDir/
    assert!(staging_path.join("WrapDir").join("mod.txt").exists());
}

#[test]
fn test_stage_one_mod_other_kind() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "OtherMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "OtherMod".to_string(),
            path: temp.path().join("nonexistent"),
            kind: ModEntryKind::Other,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "OtherMod".to_string(),
        path: temp.path().join("nonexistent"),
        kind: ModEntryKind::Other,
        metadata: None,
    };
    let result = manager.stage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    // No filesystem operation was performed
    assert!(!staging_path.join("OtherMod").exists());
    // State still transitions to Staged (prepares for next cycle)
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_stage_mods_with_mods() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    fs::create_dir(mods_path.join("ModA")).unwrap();
    fs::write(mods_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(mods_path.join("ModB")).unwrap();
    fs::write(mods_path.join("ModB").join("b.txt"), "b").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![
        ReconciledMod {
            name: "ModA".to_string(),
            status: ModStatus::Downloaded,
            source_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: mods_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            staging_entry: None,
            game_entry: None,
        },
        ReconciledMod {
            name: "ModB".to_string(),
            status: ModStatus::Downloaded,
            source_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: mods_path.join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            staging_entry: None,
            game_entry: None,
        },
    ]);

    let result = manager.stage_mods(&mut state);

    assert!(result.is_ok());
    assert!(staging_path.join("ModA").join("a.txt").exists());
    assert!(staging_path.join("ModB").join("b.txt").exists());
    assert!(state
        .snapshot()
        .iter()
        .all(|m| m.status == ModStatus::Staged));
}

#[test]
fn test_enable_mods_empty_staging() {
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    let result = manager.enable_mods(&mut state);

    assert!(result.is_ok());
}

#[test]
fn test_enable_one_mod() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Staged,
        source_entry: None,
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: staging_path.join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.enable_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("SomeMod").exists());
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_enable_one_mod_source_not_found() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    let entry = ModEntry {
        name: "Nonexistent".to_string(),
        path: staging_path.join("Nonexistent"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };

    let result = manager.enable_one_mod(&entry, &mut state);
    assert!(result.is_err());
}

#[test]
fn test_enable_mods_with_mods() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModA")).unwrap();
    fs::write(staging_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![
        ReconciledMod {
            name: "ModA".to_string(),
            status: ModStatus::Staged,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: staging_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: None,
        },
        ReconciledMod {
            name: "ModB".to_string(),
            status: ModStatus::Staged,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: staging_path.join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: None,
        },
    ]);

    let result = manager.enable_mods(&mut state);

    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("ModA").is_symlink());
    assert!(game_path.join("Mods").join("ModB").is_symlink());
    assert!(state
        .snapshot()
        .iter()
        .all(|m| m.status == ModStatus::Enabled));
}

#[test]
fn test_unstage_one_mod_dir() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Staged,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: mods_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: staging_path.join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.unstage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(!staging_path.join("SomeMod").exists());
    assert!(mods_path.join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_unstage_one_mod_not_in_downloads() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Staged,
        source_entry: None,
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: staging_path.join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.unstage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(!staging_path.join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_unstage_one_mod_nonexistent_path() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    let entry = ModEntry {
        name: "NonExistentMod".to_string(),
        path: staging_path.join("NonExistentMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.unstage_one_mod(&entry, &mut state);

    assert!(result.is_ok());
}

#[test]
fn test_unstage_mods_batch() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();
    fs::write(mods_path.join("ModA").join("a.txt"), "a").unwrap();

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModA")).unwrap();
    fs::write(staging_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![
        ReconciledMod {
            name: "ModA".to_string(),
            status: ModStatus::Staged,
            source_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: mods_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            staging_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: staging_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: None,
        },
        ReconciledMod {
            name: "ModB".to_string(),
            status: ModStatus::Staged,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: staging_path.join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: None,
        },
    ]);

    let result = manager.unstage_mods(&mut state);

    assert!(result.is_ok());
    assert!(!staging_path.join("ModA").exists());
    assert!(!staging_path.join("ModB").exists());
    assert!(mods_path.join("ModA").exists()); // source kept
                                              // ModA: exists in source → Downloaded; ModB: no source → removed
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].name, "ModA");
    assert_eq!(state.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_disable_one_mod_with_staging() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("mod.txt"), "content").unwrap();
    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(
        staging_path.join("SomeMod"),
        game_path.join("Mods").join("SomeMod"),
    )
    .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: None,
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: game_path.join("Mods").join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: game_path.join("Mods").join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.disable_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_disable_one_mod_not_in_staging_but_in_downloads() {
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::write(mods_path.join("SomeMod").join("mod.txt"), "content").unwrap();
    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(
        mods_path.join("SomeMod"),
        game_path.join("Mods").join("SomeMod"),
    )
    .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: mods_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: game_path.join("Mods").join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: game_path.join("Mods").join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.disable_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_disable_one_mod_only_in_game_mods() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    let orphan_target = temp.path().join("orphan_mod");
    fs::create_dir(&orphan_target).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(&orphan_target, game_path.join("Mods").join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: None,
        staging_entry: None,
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: game_path.join("Mods").join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
    }]);

    let entry = ModEntry {
        name: "SomeMod".to_string(),
        path: game_path.join("Mods").join("SomeMod"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };
    let result = manager.disable_one_mod(&entry, &mut state);

    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_disable_one_mod_nonexistent_game_mod() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    let entry = ModEntry {
        name: "NonExistent".to_string(),
        path: game_path.join("Mods").join("NonExistent"),
        kind: ModEntryKind::Directory,
        metadata: None,
    };

    let result = manager.disable_one_mod(&entry, &mut state);

    // path doesn't exist → deactivate skipped
    // mod not in staging or downloads → state.remove is no-op
    assert!(result.is_ok());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_disable_mods_with_mods() {
    let temp = TempDir::new().unwrap();
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModA")).unwrap();
    fs::write(staging_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();

    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(
        staging_path.join("ModA"),
        game_path.join("Mods").join("ModA"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        staging_path.join("ModB"),
        game_path.join("Mods").join("ModB"),
    )
    .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);
    let mut state = ModState::from_vec(vec![
        ReconciledMod {
            name: "ModA".to_string(),
            status: ModStatus::Enabled,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: staging_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: game_path.join("Mods").join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
        },
        ReconciledMod {
            name: "ModB".to_string(),
            status: ModStatus::Enabled,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: staging_path.join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: game_path.join("Mods").join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
        },
    ]);

    let result = manager.disable_mods(&mut state);

    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("ModA").exists());
    assert!(!game_path.join("Mods").join("ModB").exists());
    // Both are in staging → fall back to Staged
    assert!(state
        .snapshot()
        .iter()
        .all(|m| m.status == ModStatus::Staged));
}

#[test]
fn test_sync_all_full_pipeline() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::write(mods_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);

    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: mods_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let result = manager.sync_all(&mut state);

    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].status, ModStatus::Staged);
}

#[test]
fn test_sync_all_staged_to_enabled() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::write(staging_path.join("SomeMod").join("mod.txt"), "content").unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);

    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Staged,
        source_entry: None,
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: None,
    }]);

    let result = manager.sync_all(&mut state);

    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_sync_all_enabled_skipped() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(
        staging_path.join("SomeMod"),
        game_path.join("Mods").join("SomeMod"),
    )
    .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);

    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: None,
        staging_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: staging_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: game_path.join("Mods").join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
    }]);

    let result = manager.sync_all(&mut state);

    assert!(result.is_ok());
    // Symlink still intact, state unchanged
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_sync_all_multiple_statuses() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    // ModA: Downloaded → needs staging
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();
    fs::write(mods_path.join("ModA").join("a.txt"), "a").unwrap();

    // ModB: Staged → needs enabling
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();

    // ModC: Enabled → no-op
    fs::create_dir(staging_path.join("ModC")).unwrap();
    fs::write(staging_path.join("ModC").join("c.txt"), "c").unwrap();
    fs::create_dir_all(game_path.join("Mods")).unwrap();
    std::os::unix::fs::symlink(
        staging_path.join("ModC"),
        game_path.join("Mods").join("ModC"),
    )
    .unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(game_path.clone());
    let manager = SyncManager::new(game, config);

    let mut state = ModState::from_vec(vec![
        ReconciledMod {
            name: "ModA".to_string(),
            status: ModStatus::Downloaded,
            source_entry: Some(ModEntry {
                name: "ModA".to_string(),
                path: mods_path.join("ModA"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            staging_entry: None,
            game_entry: None,
        },
        ReconciledMod {
            name: "ModB".to_string(),
            status: ModStatus::Staged,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModB".to_string(),
                path: staging_path.join("ModB"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: None,
        },
        ReconciledMod {
            name: "ModC".to_string(),
            status: ModStatus::Enabled,
            source_entry: None,
            staging_entry: Some(ModEntry {
                name: "ModC".to_string(),
                path: staging_path.join("ModC"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
            game_entry: Some(ModEntry {
                name: "ModC".to_string(),
                path: game_path.join("Mods").join("ModC"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
        },
    ]);

    let result = manager.sync_all(&mut state);

    assert!(result.is_ok());
    // ModA staged
    assert!(staging_path.join("ModA").join("a.txt").exists());
    // ModB enabled
    assert!(game_path.join("Mods").join("ModB").is_symlink());
    // ModC still enabled
    assert!(game_path.join("Mods").join("ModC").is_symlink());

    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 3);
    // sync_all only advances one status per call
    assert_eq!(
        snapshot
            .iter()
            .map(|m| (&*m.name, m.status))
            .collect::<Vec<_>>(),
        vec![
            ("ModA", ModStatus::Staged),  // Downloaded → Staged
            ("ModB", ModStatus::Enabled), // Staged → Enabled
            ("ModC", ModStatus::Enabled), // Enabled → no-op
        ]
    );
}

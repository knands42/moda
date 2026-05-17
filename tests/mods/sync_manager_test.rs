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
    // Given: an empty mods folder and a default state
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    // When: staging all mods
    let result = manager.stage_mods(&mut state);

    // Then: it succeeds without error
    assert!(result.is_ok());
}

#[test]
fn test_stage_one_mod_dir() {
    // Given: a directory mod in the source folder and a state tracking it as Downloaded
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

    // When: staging the directory mod
    let result = manager.stage_one_mod(&entry, &mut state);

    // Then: the mod is copied to staging and state transitions to Staged
    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_stage_one_mod_zip() {
    // Given: a flat zip (no wrapping dir) in the source folder
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

    // When: staging the zip mod
    let result = manager.stage_one_mod(&entry, &mut state);

    // Then: the zip is extracted into a folder named after the mod (without .zip)
    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_stage_one_mod_zip_with_wrap_directory() {
    // Given: a zip with a wrapping top-level directory (WrapDir/)
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
    // State as reconcile would produce: effective name is the wrapping dir name
    let mut state = ModState::from_vec(vec![ReconciledMod {
        name: "WrapDir".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "WrapDir".to_string(),
            path: zip_path.clone(),
            kind: ModEntryKind::ZipArchive,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
    }]);

    let entry = ModEntry {
        name: "WrapDir".to_string(),
        path: zip_path,
        kind: ModEntryKind::ZipArchive,
        metadata: None,
    };

    // When: staging the wrapping zip
    let result = manager.stage_one_mod(&entry, &mut state);

    // Then: the zip extracts into staging root, and state is updated with the wrap dir name
    assert!(result.is_ok());
    assert!(staging_path.join("WrapDir").join("mod.txt").exists());
    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].name, "WrapDir");
    assert_eq!(snapshot[0].status, ModStatus::Staged);
    // Ensure the mod named after the zip file is NOT in state (only the wrap dir name)
    assert!(state.get_mod("WrapDir").is_some());
    assert!(state.get_mod("SomeMod").is_none());
}

#[test]
fn test_stage_mods_with_mods() {
    // Given: multiple directory mods in the source folder with tracked state
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

    // When: batch-staging all mods
    let result = manager.stage_mods(&mut state);

    // Then: all mods are copied to staging and state reflects Staged
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
    // Given: an empty staging folder and a default state
    let temp = TempDir::new().unwrap();
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let game = StardewValley::new(temp.path().join("game"));
    let manager = SyncManager::new(game, config);
    let mut state = ModState::default();

    // When: enabling all mods (none exist in staging)
    let result = manager.enable_mods(&mut state);

    // Then: it succeeds without error
    assert!(result.is_ok());
}

#[test]
fn test_enable_one_mod() {
    // Given: a staged mod in the staging folder
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

    // When: enabling the mod
    let result = manager.enable_one_mod(&entry, &mut state);

    // Then: a symlink is created in the game mods folder and state transitions to Enabled
    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("SomeMod").exists());
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_enable_one_mod_source_not_found() {
    // Given: a mod entry pointing to a non-existent staging path
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

    // When: trying to enable a non-existent mod
    let result = manager.enable_one_mod(&entry, &mut state);

    // Then: an error is returned (Enabler::activate requires source to exist)
    assert!(result.is_err());
}

#[test]
fn test_enable_mods_with_mods() {
    // Given: multiple staged mods in the staging folder
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

    // When: batch-enabling all staged mods
    let result = manager.enable_mods(&mut state);

    // Then: all mods get symlinks in game mods and state shows Enabled
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
    // Given: a staged mod that also exists in the source folder
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

    // When: unstaging the mod
    let result = manager.unstage_one_mod(&entry, &mut state);

    // Then: staging is removed, source is preserved, state reverts to Downloaded
    assert!(result.is_ok());
    assert!(!staging_path.join("SomeMod").exists());
    assert!(mods_path.join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_unstage_one_mod_not_in_downloads() {
    // Given: a staged mod that was removed from the source folder
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

    // When: unstaging (source no longer present)
    let result = manager.unstage_one_mod(&entry, &mut state);

    // Then: staging is removed and the mod is fully removed from state
    assert!(result.is_ok());
    assert!(!staging_path.join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_unstage_one_mod_nonexistent_path() {
    // Given: a mod entry whose staging path does not exist on disk
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

    // When: trying to unstage a non-existent mod
    let result = manager.unstage_one_mod(&entry, &mut state);

    // Then: it succeeds gracefully (no-op)
    assert!(result.is_ok());
}

#[test]
fn test_unstage_mods_batch() {
    // Given: three mods — one staged with source, one staged without source,
    //        one enabled with source and staging
    let temp = TempDir::new().unwrap();
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    let game_path = temp.path().join("game");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();
    fs::write(mods_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(mods_path.join("ModC")).unwrap();
    fs::write(mods_path.join("ModC").join("c.txt"), "c").unwrap();

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModA")).unwrap();
    fs::write(staging_path.join("ModA").join("a.txt"), "a").unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();
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
        ReconciledMod {
            name: "ModC".to_string(),
            status: ModStatus::Enabled,
            source_entry: Some(ModEntry {
                name: "ModC".to_string(),
                path: mods_path.join("ModC"),
                kind: ModEntryKind::Directory,
                metadata: None,
            }),
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

    // When: batch-unstaging all mods
    let result = manager.unstage_mods(&mut state);

    // Then: ModA reverts to Downloaded, ModB is removed entirely,
    //       ModC's symlink is removed and reverts to Downloaded
    assert!(result.is_ok());
    assert!(!staging_path.join("ModA").exists());
    assert!(!staging_path.join("ModB").exists());
    assert!(!staging_path.join("ModC").exists());
    assert!(!game_path.join("Mods").join("ModC").exists());
    assert!(mods_path.join("ModA").exists());
    assert!(mods_path.join("ModC").exists());
    let mut snapshot = state.snapshot();
    snapshot.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[0].name, "ModA");
    assert_eq!(snapshot[0].status, ModStatus::Downloaded);
    assert_eq!(snapshot[1].name, "ModC");
    assert_eq!(snapshot[1].status, ModStatus::Downloaded);
}

#[test]
fn test_disable_one_mod_with_staging() {
    // Given: an enabled mod that also exists in the staging folder
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

    // When: disabling the mod
    let result = manager.disable_one_mod(&entry, &mut state);

    // Then: the symlink is removed and state reverts to Staged
    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Staged);
}

#[test]
fn test_disable_one_mod_not_in_staging_but_in_downloads() {
    // Given: an enabled mod symlinked directly from source (no staging)
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

    // When: disabling (mod not in staging but in source)
    let result = manager.disable_one_mod(&entry, &mut state);

    // Then: symlink removed and state reverts to Downloaded
    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 1);
    assert_eq!(state.snapshot()[0].status, ModStatus::Downloaded);
}

#[test]
fn test_disable_one_mod_only_in_game_mods() {
    // Given: an orphan enabled mod (game symlink with no source or staging)
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

    // When: disabling the orphan mod
    let result = manager.disable_one_mod(&entry, &mut state);

    // Then: symlink removed and mod fully removed from state
    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("SomeMod").exists());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_disable_one_mod_nonexistent_game_mod() {
    // Given: a mod entry whose game mod path does not exist on disk
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

    // When: trying to disable a non-existent game mod
    let result = manager.disable_one_mod(&entry, &mut state);

    // Then: deactivation is skipped, state.remove is a no-op, returns Ok
    assert!(result.is_ok());
    assert_eq!(state.snapshot().len(), 0);
}

#[test]
fn test_disable_mods_batch() {
    // Given: multiple enabled mods with symlinks in the game mods folder
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

    // When: batch-disabling all mods
    let result = manager.disable_mods(&mut state);

    // Then: all symlinks are removed and state shows Staged
    assert!(result.is_ok());
    assert!(!game_path.join("Mods").join("ModA").exists());
    assert!(!game_path.join("Mods").join("ModB").exists());
    assert!(state
        .snapshot()
        .iter()
        .all(|m| m.status == ModStatus::Staged));
}

#[test]
fn test_sync_all_full_pipeline() {
    // Given: a mod in Downloaded state with source present
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

    // When: running sync_all
    let result = manager.sync_all(&mut state);

    // Then: the mod is staged (one status advancement) and state reflects Staged
    assert!(result.is_ok());
    assert!(staging_path.join("SomeMod").join("mod.txt").exists());
    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].status, ModStatus::Staged);
}

#[test]
fn test_sync_all_staged_to_enabled() {
    // Given: a mod in Staged state with staging entry present
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

    // When: running sync_all
    let result = manager.sync_all(&mut state);

    // Then: the mod is enabled (symlink created) and state reflects Enabled
    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_sync_all_enabled_skipped() {
    // Given: a mod already in Enabled state with game entry
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

    // When: running sync_all
    let result = manager.sync_all(&mut state);

    // Then: the enabled mod is skipped, symlink and state are unchanged
    assert!(result.is_ok());
    assert!(game_path.join("Mods").join("SomeMod").is_symlink());
    assert_eq!(state.snapshot()[0].status, ModStatus::Enabled);
}

#[test]
fn test_sync_all_multiple_statuses() {
    // Given: three mods in different states — Downloaded, Staged, Enabled
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();
    fs::write(mods_path.join("ModA").join("a.txt"), "a").unwrap();

    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();
    fs::write(staging_path.join("ModB").join("b.txt"), "b").unwrap();

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

    // When: running sync_all
    let result = manager.sync_all(&mut state);

    // Then: each mod advances exactly one status
    assert!(result.is_ok());
    assert!(staging_path.join("ModA").join("a.txt").exists());
    assert!(game_path.join("Mods").join("ModB").is_symlink());
    assert!(game_path.join("Mods").join("ModC").is_symlink());

    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 3);
    assert_eq!(
        snapshot
            .iter()
            .map(|m| (&*m.name, m.status))
            .collect::<Vec<_>>(),
        vec![
            ("ModA", ModStatus::Staged),  // Downloaded -> Staged
            ("ModB", ModStatus::Enabled), // Staged -> Enabled
            ("ModC", ModStatus::Enabled), // Enabled -> no-op
        ]
    );
}

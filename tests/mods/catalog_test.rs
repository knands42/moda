use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use crate::mods::test_util::{create_zip, make_config};
use moda::mods::catalog::Catalog;
use moda::mods::repository::{ModRepository, TursoModRepository};
use moda::mods::types::{ModEntry, ModEntryKind, ModStatus, ReconciledMod};

// --- reconcile_from_filesystem: empty directory ---

#[test]
fn test_reconcile_from_filesystem_empty() {
    // Given: an empty game Mods directory
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then
    assert!(result.snapshot().is_empty());
}

// --- reconcile_from_filesystem: non-symlink game dir treated as Staged ---

#[test]
fn test_reconcile_from_filesystem_non_symlink_is_staged() {
    // Given: mod present in mods, staging, and game dirs — but game entry
    //        is a real directory (not a symlink) and no DB record exists
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir(game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: game_entry discarded (not symlink, no DB), mod is Staged
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Staged);

    let src = m.source_entry.as_ref().unwrap();
    assert_eq!(src.kind, ModEntryKind::Directory);
    assert!(src.path.ends_with("mods/stardew_valley/SomeMod"));

    let stg = m.staging_entry.as_ref().unwrap();
    assert_eq!(stg.kind, ModEntryKind::Directory);
    assert!(stg.path.ends_with("staging/stardew_valley/SomeMod"));

    assert!(m.game_entry.is_none());
}

// --- reconcile_from_filesystem: zip variants (flat, wrapped, diff-name, mixed) ---

#[test]
fn test_reconcile_from_filesystem_zip_variants() {
    // Given: four zip files in mods/ with different internal structures
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();

    // Flat zip — files at root, effective name = strip_zip_ext
    create_zip(&mods_path.join("FlatMod.zip"), &["mod.txt"]);

    // Wrapped zip — single top-level dir, effective name = wrap dir
    create_zip(&mods_path.join("WrappedMod.zip"), &["WrappedDir/mod.txt"]);

    // Zip whose wrap dir differs from the filename
    create_zip(
        &mods_path.join("DiffName.zip"),
        &["DifferentName-v2/mod.txt", "DifferentName-v2/sub/asset.dat"],
    );

    // Mixed zip — wrapping dir + files at root, no single wrap → strip_zip_ext
    create_zip(
        &mods_path.join("Mixed-1.0.0.zip"),
        &["Mixed/mod.txt", "readme.md"],
    );

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: all four are Downloaded with correct effective names
    assert_eq!(result.snapshot().len(), 4);

    for m in result.snapshot() {
        assert_eq!(m.status, ModStatus::Downloaded);
        assert!(m.source_entry.is_some());
        assert!(m.staging_entry.is_none());
        assert!(m.game_entry.is_none());
    }

    let snapshot = result.snapshot();

    // FlatMod — effective name from zip stem
    let flat = snapshot.iter().find(|m| m.name == "FlatMod").unwrap();
    assert_eq!(
        flat.source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
    assert!(flat
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/FlatMod.zip"));

    // WrappedDir — effective name from wrapping directory
    let wrapped = snapshot.iter().find(|m| m.name == "WrappedDir").unwrap();
    assert_eq!(
        wrapped.source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
    assert!(wrapped
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/WrappedMod.zip"));

    // DifferentName-v2 — wraps in DifferentName dir, zip filename is DiffName
    let diff = snapshot
        .iter()
        .find(|m| m.name == "DifferentName-v2")
        .unwrap();
    assert_eq!(
        diff.source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
    assert!(diff
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/DiffName.zip"));

    // Mixed-1.0.0 — files at root + subdir → no single wrap, name from stem
    let mixed = snapshot.iter().find(|m| m.name == "Mixed-1.0.0").unwrap();
    assert_eq!(
        mixed.source_entry.as_ref().unwrap().kind,
        ModEntryKind::ZipArchive
    );
    assert!(mixed
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/Mixed-1.0.0.zip"));
}

// --- reconcile_from_filesystem: enabled via direct copy, backed by DB entry ---

#[test]
fn test_reconcile_from_filesystem_with_db_game_entry() {
    // Given: dirs in mods + staging + game (not a symlink), DB pre-populated
    //        as Enabled with a game_entry
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    fs::create_dir(game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();
    let reconciled = ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: None,
        staging_entry: None,
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: game_path.join("SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        register_id: "stardew_valley".to_string(),
    };
    repo.upsert_mod("stardew_valley", &reconciled).unwrap();
    drop(repo);

    // When: Catalog opens the same DB and reconciles
    let catalog = Catalog::new(config, "stardew_valley");
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: mod is Enabled with all three entries from filesystem + DB
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Enabled);

    assert_eq!(
        m.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(m
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/SomeMod"));

    assert_eq!(
        m.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(m
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/SomeMod"));

    assert_eq!(m.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(m
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/SomeMod"));
}

// --- reconcile_from_filesystem: multiple mixed states (Downloaded / Staged / Enabled) ---

#[test]
fn test_reconcile_from_filesystem_multiple_mixed_states() {
    // Given: three mods in different states:
    //   ModA — source only (Downloaded)
    //   ModB — source + staging (Staged)
    //   ModC — source + staging + symlink (Enabled)
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();

    fs::create_dir(mods_path.join("ModA")).unwrap();

    fs::create_dir(mods_path.join("ModB")).unwrap();
    fs::create_dir(staging_path.join("ModB")).unwrap();

    fs::create_dir(mods_path.join("ModC")).unwrap();
    fs::create_dir(staging_path.join("ModC")).unwrap();
    std::os::unix::fs::symlink(staging_path.join("ModC"), game_path.join("ModC")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();
    let snapshot = result.snapshot();

    // Then
    assert_eq!(snapshot.len(), 3);

    // ModA: Downloaded — source only
    let a = snapshot.iter().find(|m| m.name == "ModA").unwrap();
    assert_eq!(a.status, ModStatus::Downloaded);
    assert_eq!(
        a.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(a
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/ModA"));
    assert!(a.staging_entry.is_none());
    assert!(a.game_entry.is_none());

    // ModB: Staged — source + staging, no game
    let b = snapshot.iter().find(|m| m.name == "ModB").unwrap();
    assert_eq!(b.status, ModStatus::Staged);
    assert_eq!(
        b.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(b
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/ModB"));
    assert_eq!(
        b.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(b
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/ModB"));
    assert!(b.game_entry.is_none());

    // ModC: Enabled — source + staging + symlink
    let c = snapshot.iter().find(|m| m.name == "ModC").unwrap();
    assert_eq!(c.status, ModStatus::Enabled);
    assert_eq!(
        c.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(c
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/ModC"));
    assert_eq!(
        c.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(c
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/ModC"));
    assert_eq!(c.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(c
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/ModC"));
}

// --- reconcile_from_filesystem: enabled with no source (dangling symlink) ---

#[test]
fn test_reconcile_from_filesystem_enabled_no_source() {
    // Given: symlink in game dir whose target does not exist
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    std::os::unix::fs::symlink(mods_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: Enabled with only game_entry — no source or staging
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Enabled);
    assert!(m.staging_entry.is_none());
    assert!(m.source_entry.is_none());
    assert_eq!(m.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(m
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/SomeMod"));
}

// --- reconcile_from_filesystem: enabled with source but no staging ---

#[test]
fn test_reconcile_from_filesystem_enabled_no_staging() {
    // Given: source dir + symlink in game, no staging
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(mods_path.join("SomeMod")).unwrap();
    std::os::unix::fs::symlink(mods_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: Enabled — source and game_entry present, staging absent
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Enabled);
    assert_eq!(
        m.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(m
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/SomeMod"));
    assert!(m.staging_entry.is_none());
    assert_eq!(m.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(m
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/SomeMod"));
}

// --- reconcile_from_filesystem: orphan enabled (symlink outside managed dirs) ---

#[test]
fn test_reconcile_from_filesystem_orphan_enabled() {
    // Given: symlink in game dir pointing outside mods/staging
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let orphan_target = temp.path().join("orphan_target");
    fs::create_dir(&orphan_target).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    std::os::unix::fs::symlink(&orphan_target, game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: Enabled — only game_entry, no source or staging
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Enabled);
    assert!(m.source_entry.is_none());
    assert!(m.staging_entry.is_none());
    assert_eq!(m.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(m
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/SomeMod"));
}

// --- reconcile_from_filesystem: orphan staged (staging without source) ---

#[test]
fn test_reconcile_from_filesystem_orphan_staged() {
    // Given: staging dir alone, no source, no game entry
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: Staged — no source, no game_entry, just staging
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Staged);
    assert!(m.source_entry.is_none());
    assert_eq!(
        m.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(m
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/SomeMod"));
    assert!(m.game_entry.is_none());
}

// --- reconcile_from_filesystem: modified (source newer than staging) ---

#[test]
fn test_reconcile_from_filesystem_modified() {
    // Given:
    //   ModA — was enabled (staging + symlink), source re-downloaded (newer)
    //   ModB — was staged, source re-downloaded (newer), no game entry
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp
        .path()
        .join(".moda")
        .join("mods")
        .join("stardew_valley");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();

    // ModA: staging created first, then source (newer), then symlink
    fs::create_dir(staging_path.join("ModA")).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();

    std::os::unix::fs::symlink(staging_path.join("ModA"), game_path.join("ModA")).unwrap();

    // ModB: staging created first, then source (newer)
    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir(staging_path.join("ModB")).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir(mods_path.join("ModB")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();
    let snapshot = result.snapshot();

    // Then: both mods are Modified (source mtime > staging mtime)
    assert_eq!(snapshot.len(), 2);

    // ModA: source + staging + symlink all present
    let a = snapshot.iter().find(|m| m.name == "ModA").unwrap();
    assert_eq!(a.status, ModStatus::Modified);
    assert_eq!(
        a.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(a
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/ModA"));
    assert_eq!(
        a.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(a
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/ModA"));
    assert_eq!(a.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(a
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/ModA"));

    // ModB: source + staging, no game entry
    let b = snapshot.iter().find(|m| m.name == "ModB").unwrap();
    assert_eq!(b.status, ModStatus::Modified);
    assert_eq!(
        b.source_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(b
        .source_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("mods/stardew_valley/ModB"));
    assert_eq!(
        b.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(b
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/ModB"));
    assert!(b.game_entry.is_none());
}

// --- reconcile_from_filesystem: enabled with staging only (no source) ---

#[test]
fn test_reconcile_from_filesystem_enabled_staging_only() {
    // Given: staging dir + symlink to it, no source
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let staging_path = temp
        .path()
        .join(".moda")
        .join("staging")
        .join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();
    std::os::unix::fs::symlink(staging_path.join("SomeMod"), game_path.join("SomeMod")).unwrap();

    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let result = catalog.reconcile_from_filesystem(&game_path).unwrap();

    // Then: Enabled — staging + symlink, no source
    assert_eq!(result.snapshot().len(), 1);
    let m = &result.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Enabled);
    assert!(m.source_entry.is_none());
    assert_eq!(
        m.staging_entry.as_ref().unwrap().kind,
        ModEntryKind::Directory
    );
    assert!(m
        .staging_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("staging/stardew_valley/SomeMod"));
    assert_eq!(m.game_entry.as_ref().unwrap().kind, ModEntryKind::Directory);
    assert!(m
        .game_entry
        .as_ref()
        .unwrap()
        .path
        .ends_with("game/Mods/SomeMod"));
}

// --- reconcile_from_db: empty database ---

#[test]
fn test_reconcile_from_db_empty() {
    // Given: a fresh DB with no mods
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let state = catalog.reconcile_from_db().unwrap();

    // Then
    assert!(state.snapshot().is_empty());
}

// --- reconcile_from_db: single mod with Downloaded status ---

#[test]
fn test_reconcile_from_db_single_mod() {
    // Given: one mod persisted as Downloaded with a source entry
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();
    let input = ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: PathBuf::from("/mods/stardew_valley/SomeMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
        register_id: "stardew_valley".to_string(),
    };
    repo.upsert_mod("stardew_valley", &input).unwrap();
    drop(repo);

    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let state = catalog.reconcile_from_db().unwrap();

    // Then
    assert_eq!(state.snapshot().len(), 1);
    let m = &state.snapshot()[0];
    assert_eq!(m.name, "SomeMod");
    assert_eq!(m.status, ModStatus::Downloaded);
    assert!(m.source_entry.is_some());
    assert!(m.staging_entry.is_none());
    assert!(m.game_entry.is_none());
    assert_eq!(m.register_id, "stardew_valley");
}

// --- reconcile_from_db: mod with all three entries populated ---

#[test]
fn test_reconcile_from_db_with_all_entries() {
    // Given: one mod persisted as Enabled with source, staging, and game entries
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();
    let input = ReconciledMod {
        name: "FullMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: Some(ModEntry {
            name: "FullMod".to_string(),
            path: PathBuf::from("/mods/stardew_valley/FullMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: Some(ModEntry {
            name: "FullMod".to_string(),
            path: PathBuf::from("/staging/stardew_valley/FullMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        game_entry: Some(ModEntry {
            name: "FullMod".to_string(),
            path: PathBuf::from("/game/Mods/FullMod"),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        register_id: "stardew_valley".to_string(),
    };
    repo.upsert_mod("stardew_valley", &input).unwrap();
    drop(repo);

    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let state = catalog.reconcile_from_db().unwrap();
    let snapshot = state.snapshot();

    // Then
    assert_eq!(snapshot.len(), 1);
    let m = &snapshot[0];
    assert_eq!(m.name, "FullMod");
    assert_eq!(m.status, ModStatus::Enabled);
    assert_eq!(m.register_id, "stardew_valley");

    let src = m.source_entry.as_ref().unwrap();
    assert_eq!(src.name, "FullMod");
    assert!(src.path.ends_with("mods/stardew_valley/FullMod"));
    assert_eq!(src.kind, ModEntryKind::Directory);

    let stg = m.staging_entry.as_ref().unwrap();
    assert_eq!(stg.name, "FullMod");
    assert!(stg.path.ends_with("staging/stardew_valley/FullMod"));
    assert_eq!(stg.kind, ModEntryKind::Directory);

    let game = m.game_entry.as_ref().unwrap();
    assert_eq!(game.name, "FullMod");
    assert!(game.path.ends_with("game/Mods/FullMod"));
    assert_eq!(game.kind, ModEntryKind::Directory);
}

// --- reconcile_from_db: isolation between game registry IDs ---

#[test]
fn test_reconcile_from_db_isolation() {
    // Given: mod inserted under "game_a", none under "game_b"
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();
    let input = ReconciledMod {
        name: "ModA".to_string(),
        status: ModStatus::Downloaded,
        source_entry: None,
        staging_entry: None,
        game_entry: None,
        register_id: "game_a".to_string(),
    };
    repo.upsert_mod("game_a", &input).unwrap();
    drop(repo);

    let catalog_b = Catalog::new(config, "game_b");

    // When
    let state = catalog_b.reconcile_from_db().unwrap();

    // Then: game_b sees no mods
    assert!(state.snapshot().is_empty());
}

// --- reconcile_from_db: preserves all status variants ---

#[test]
fn test_reconcile_from_db_preserves_all_statuses() {
    // Given: one mod per status variant
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    for (name, status) in [
        ("ModDownloaded", ModStatus::Downloaded),
        ("ModStaged", ModStatus::Staged),
        ("ModEnabled", ModStatus::Enabled),
        ("ModModified", ModStatus::Modified),
    ] {
        let m = ReconciledMod {
            name: name.to_string(),
            status,
            source_entry: None,
            staging_entry: None,
            game_entry: None,
            register_id: "stardew_valley".to_string(),
        };
        repo.upsert_mod("stardew_valley", &m).unwrap();
    }
    drop(repo);

    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let state = catalog.reconcile_from_db().unwrap();
    let snapshot = state.snapshot();

    // Then
    assert_eq!(snapshot.len(), 4);
    for m in &snapshot {
        match m.name.as_str() {
            "ModDownloaded" => assert_eq!(m.status, ModStatus::Downloaded),
            "ModStaged" => assert_eq!(m.status, ModStatus::Staged),
            "ModEnabled" => assert_eq!(m.status, ModStatus::Enabled),
            "ModModified" => assert_eq!(m.status, ModStatus::Modified),
            _ => panic!("unexpected mod: {}", m.name),
        }
    }
}

// --- reconcile_from_db: after remove, mod is gone ---

#[test]
fn test_reconcile_from_db_after_remove() {
    // Given: insert then remove a mod
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();
    let input = ReconciledMod {
        name: "TempMod".to_string(),
        status: ModStatus::Staged,
        source_entry: None,
        staging_entry: None,
        game_entry: None,
        register_id: "stardew_valley".to_string(),
    };
    repo.upsert_mod("stardew_valley", &input).unwrap();
    repo.remove_mod("stardew_valley", "TempMod").unwrap();
    drop(repo);

    let catalog = Catalog::new(config, "stardew_valley");

    // When
    let state = catalog.reconcile_from_db().unwrap();

    // Then
    assert!(state.snapshot().is_empty());
}

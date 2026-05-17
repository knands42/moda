use moda::games::StardewValley;
use moda::mods::catalog::{Catalog, ModEntryKind, ModStatus};
use std::fs;
use tempfile::TempDir;

use crate::mods::test_util::{create_zip, make_config};

#[test]
fn test_reconcile_empty() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

    assert!(result.snapshot().is_empty());
}

#[test]
fn test_reconcile_enabled_failed_if_not_symlink_mod() {
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
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

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

#[test]
fn test_reconcile_zip_variants() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
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

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

    assert_eq!(result.snapshot().len(), 4);

    for m in result.snapshot() {
        assert_eq!(m.status, ModStatus::Downloaded);
        assert!(m.source_entry.is_some());
        assert!(m.staging_entry.is_none());
        assert!(m.game_entry.is_none());
    }

    let snapshot = result.snapshot();

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

#[test]
fn test_reconcile_multiple_mixed_states() {
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
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();
    let snapshot = result.snapshot();

    assert_eq!(snapshot.len(), 3);

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

#[test]
fn test_reconcile_enabled_mod_without_staging() {
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
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

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

#[test]
fn test_reconcile_orphan_enabled() {
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
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

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

#[test]
fn test_reconcile_orphan_staged() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&staging_path).unwrap();
    fs::create_dir(staging_path.join("SomeMod")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

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

#[test]
fn test_reconcile_modified() {
    let temp = TempDir::new().unwrap();
    let game_path = temp.path().join("game").join("Mods");
    let mods_path = temp.path().join("mods").join("stardew_valley");
    let staging_path = temp.path().join("staging").join("stardew_valley");
    fs::create_dir_all(&game_path).unwrap();
    fs::create_dir_all(&staging_path).unwrap();

    // ModA: source newer than staging + game symlink (was enabled)
    fs::create_dir(staging_path.join("ModA")).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir_all(&mods_path).unwrap();
    fs::create_dir(mods_path.join("ModA")).unwrap();

    std::os::unix::fs::symlink(staging_path.join("ModA"), game_path.join("ModA")).unwrap();

    // ModB: source newer than staging, no game entry (was only staged)
    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir(staging_path.join("ModB")).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::create_dir(mods_path.join("ModB")).unwrap();

    let config = make_config(
        temp.path().join("mods").to_str().unwrap(),
        temp.path().join("staging").to_str().unwrap(),
    );
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();
    let snapshot = result.snapshot();

    assert_eq!(snapshot.len(), 2);

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

#[test]
fn test_reconcile_enabled_with_staging_only_no_source() {
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
    let catalog: Catalog<StardewValley> = Catalog::new(config);

    let result = catalog.reconcile(&game_path).unwrap();

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

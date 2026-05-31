use crate::mods::test_util::{make_config, new_repo};
use moda::mods::repository::{ModRepository, TursoModRepository};
use moda::mods::types::{ModEntry, ModEntryKind, ModStatus, ReconciledMod};
use tempfile::TempDir;

fn make_reconciled(name: &str, status: ModStatus) -> ReconciledMod {
    ReconciledMod {
        name: name.to_string(),
        status,
        source_entry: None,
        staging_entry: None,
        game_entry: None,
        register_id: name.to_lowercase(),
    }
}

#[test]
fn test_new_repo_get_mods_empty() {
    let repo = new_repo();
    let mods = repo.get_mods("test_game").unwrap();
    assert!(mods.is_empty());
}

#[test]
fn test_new_repo_upsert_persists() {
    let repo = new_repo();
    let rm = make_reconciled("SomeMod", ModStatus::Downloaded);

    let result = repo.upsert_mod("test_game", &rm);
    assert!(result.is_ok());

    let mods = repo.get_mods("test_game").unwrap();
    assert_eq!(mods.len(), 1);
}

#[test]
fn test_new_repo_remove() {
    let repo = new_repo();
    let rm = make_reconciled("SomeMod", ModStatus::Downloaded);
    repo.upsert_mod("test_game", &rm).unwrap();
    assert_eq!(repo.get_mods("test_game").unwrap().len(), 1);

    let result = repo.remove_mod("test_game", "SomeMod");
    assert!(result.is_ok());
    assert_eq!(repo.get_mods("test_game").unwrap().len(), 0);
}

// --- TursoModRepository tests ---

#[test]
fn test_turso_new_creates_database() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);

    let repo = TursoModRepository::new(&config);
    assert!(repo.is_ok());
}

#[test]
fn test_turso_get_mods_empty() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let mods = repo.get_mods("stardew_valley").unwrap();
    assert!(mods.is_empty());
}

#[test]
fn test_turso_upsert_and_get() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let rm = ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: "/tmp/moda_test/SomeMod".into(),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        staging_entry: None,
        game_entry: None,
        register_id: "SomeMod".to_lowercase(),
    };

    repo.upsert_mod("stardew_valley", &rm).unwrap();

    let mods = repo.get_mods("stardew_valley").unwrap();
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].name, "SomeMod");
    assert_eq!(mods[0].status, ModStatus::Downloaded);
    assert!(mods[0].source_entry.is_some());
    assert!(mods[0].staging_entry.is_none());
    assert!(mods[0].game_entry.is_none());
}

#[test]
fn test_turso_upsert_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let rm1 = ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Downloaded,
        source_entry: None,
        staging_entry: None,
        game_entry: None,
        register_id: "SomeMod".to_lowercase(),
    };
    repo.upsert_mod("stardew_valley", &rm1).unwrap();

    let rm2 = ReconciledMod {
        name: "SomeMod".to_string(),
        status: ModStatus::Enabled,
        source_entry: None,
        staging_entry: None,
        game_entry: Some(ModEntry {
            name: "SomeMod".to_string(),
            path: "/tmp/moda_test/SomeMod".into(),
            kind: ModEntryKind::Directory,
            metadata: None,
        }),
        register_id: "SomeMod".to_lowercase(),
    };
    repo.upsert_mod("stardew_valley", &rm2).unwrap();

    let mods = repo.get_mods("stardew_valley").unwrap();
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].status, ModStatus::Enabled);
    assert!(mods[0].game_entry.is_some());
}

#[test]
fn test_turso_game_registry_isolation() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let rm = make_reconciled("SomeMod", ModStatus::Downloaded);
    repo.upsert_mod("game_a", &rm).unwrap();
    repo.upsert_mod("game_b", &rm).unwrap();

    assert_eq!(repo.get_mods("game_a").unwrap().len(), 1);
    assert_eq!(repo.get_mods("game_b").unwrap().len(), 1);
    assert_eq!(repo.get_mods("game_c").unwrap().len(), 0);
}

#[test]
fn test_turso_remove_mod() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let rm = make_reconciled("SomeMod", ModStatus::Downloaded);
    repo.upsert_mod("stardew_valley", &rm).unwrap();
    assert_eq!(repo.get_mods("stardew_valley").unwrap().len(), 1);

    repo.remove_mod("stardew_valley", "SomeMod").unwrap();
    assert_eq!(repo.get_mods("stardew_valley").unwrap().len(), 0);
}

#[test]
fn test_turso_remove_nonexistent_is_noop() {
    let temp = TempDir::new().unwrap();
    let config = make_config(&temp);
    let repo = TursoModRepository::new(&config).unwrap();

    let result = repo.remove_mod("stardew_valley", "NonExistent");
    assert!(result.is_ok());
}

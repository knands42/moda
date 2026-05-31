use crate::mods::test_util::new_repo;
use moda::mods::types::{ModEntry, ModEntryKind, ModStatus, ReconciledMod};
use moda::mods::ModState;
use std::collections::HashMap;

fn make_entry(name: &str, kind: ModEntryKind) -> ModEntry {
    ModEntry {
        name: name.to_string(),
        path: format!("/mods/{}", name).into(),
        kind,
        metadata: None,
    }
}

fn make_state(mods: Vec<(&str, ModStatus)>) -> ModState {
    let reconciled = mods
        .into_iter()
        .map(|(name, status)| {
            (
                name.to_string(),
                ReconciledMod {
                    name: name.to_string(),
                    status,
                    source_entry: None,
                    staging_entry: None,
                    game_entry: None,
                    register_id: String::new(),
                },
            )
        })
        .collect();
    ModState::new(reconciled, new_repo())
}

fn empty_state() -> ModState {
    ModState::new(HashMap::new(), new_repo())
}

// --- empty / new ---

#[test]
fn test_new_empty() {
    let state = ModState::new(HashMap::new(), new_repo());
    assert!(state.snapshot().is_empty());
    assert!(state.get_mods().next().is_none());
}

// --- set_staged ---

#[test]
fn test_set_staged_transitions_from_downloaded() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);
    let entry = make_entry("SomeMod", ModEntryKind::Directory);

    state.set_staged(&entry);

    let m = state.get_mod("SomeMod").unwrap();
    assert_eq!(m.status, ModStatus::Staged);
    assert_eq!(m.staging_entry.as_ref().map(|e| &*e.name), Some("SomeMod"));
    assert!(m.game_entry.is_none());
    assert!(m.source_entry.is_none());
}

#[test]
fn test_set_staged_clears_game_entry() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Enabled)]);
    let entry = make_entry("SomeMod", ModEntryKind::Directory);
    state.set_staged(&entry);

    assert!(state.get_mod("SomeMod").unwrap().game_entry.is_none());
}

#[test]
fn test_set_staged_nonexistent_mod_is_noop() {
    let mut state = make_state(vec![]);
    let entry = make_entry("Ghost", ModEntryKind::Directory);

    state.set_staged(&entry); // should not panic
    assert!(state.snapshot().is_empty());
}

// --- set_downloaded ---

#[test]
fn test_set_downloaded_from_staged() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Staged)]);
    let entry = make_entry("SomeMod", ModEntryKind::Directory);

    state.set_downloaded(&entry);

    let m = state.get_mod("SomeMod").unwrap();
    assert_eq!(m.status, ModStatus::Downloaded);
    assert_eq!(m.source_entry.as_ref().map(|e| &*e.name), Some("SomeMod"));
    assert!(m.staging_entry.is_none());
    assert!(m.game_entry.is_none());
}

#[test]
fn test_set_downloaded_nonexistent_is_noop() {
    let mut state = empty_state();
    state.set_downloaded(&make_entry("Ghost", ModEntryKind::Directory));
    assert!(state.snapshot().is_empty());
}

// --- set_enabled ---

#[test]
fn test_set_enabled_from_staged() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Staged)]);
    let entry = make_entry("SomeMod", ModEntryKind::Directory);

    state.set_enabled(&entry);

    let m = state.get_mod("SomeMod").unwrap();
    assert_eq!(m.status, ModStatus::Enabled);
    assert_eq!(m.game_entry.as_ref().map(|e| &*e.name), Some("SomeMod"));
}

#[test]
fn test_set_enabled_nonexistent_is_noop() {
    let mut state = empty_state();
    state.set_enabled(&make_entry("Ghost", ModEntryKind::Directory));
    assert!(state.snapshot().is_empty());
}

// --- set_unstaged ---

#[test]
fn test_set_unstaged_clears_entries_and_reverts_to_downloaded() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Staged)]);

    state.set_unstaged("SomeMod");

    let m = state.get_mod("SomeMod").unwrap();
    assert_eq!(m.status, ModStatus::Downloaded);
    assert!(m.staging_entry.is_none());
    assert!(m.game_entry.is_none());
}

#[test]
fn test_set_unstaged_nonexistent_is_noop() {
    let mut state = empty_state();
    state.set_unstaged("Ghost");
    assert!(state.snapshot().is_empty());
}

// --- set_disabled ---

#[test]
fn test_set_disabled_from_enabled() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Enabled)]);

    state.set_disabled("SomeMod");

    let m = state.get_mod("SomeMod").unwrap();
    assert_eq!(m.status, ModStatus::Staged);
    assert!(m.game_entry.is_none());
}

#[test]
fn test_set_disabled_nonexistent_is_noop() {
    let mut state = empty_state();
    state.set_disabled("Ghost");
    assert!(state.snapshot().is_empty());
}

// --- remove ---

#[test]
fn test_remove_existing_mod() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);

    state.remove("SomeMod");

    assert!(state.get_mod("SomeMod").is_none());
    assert!(state.snapshot().is_empty());
}

#[test]
fn test_remove_nonexistent_mod_is_noop() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);

    state.remove("Ghost"); // should not panic

    assert_eq!(state.snapshot().len(), 1);
}

// --- lifecycle: full transition chain ---

#[test]
fn test_full_lifecycle() {
    let mut state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);

    // Downloaded -> Staged
    state.set_staged(&make_entry("SomeMod", ModEntryKind::Directory));
    assert_eq!(state.get_mod("SomeMod").unwrap().status, ModStatus::Staged);

    // Staged -> Enabled
    state.set_enabled(&make_entry("SomeMod", ModEntryKind::Directory));
    assert_eq!(state.get_mod("SomeMod").unwrap().status, ModStatus::Enabled);

    // Enabled -> Disabled (back to Staged)
    state.set_disabled("SomeMod");
    assert_eq!(state.get_mod("SomeMod").unwrap().status, ModStatus::Staged);

    // Staged -> Unstaged (back to Downloaded)
    state.set_unstaged("SomeMod");
    assert_eq!(
        state.get_mod("SomeMod").unwrap().status,
        ModStatus::Downloaded
    );

    // Remove entirely
    state.remove("SomeMod");
    assert!(state.get_mod("SomeMod").is_none());
}

// --- snapshot / get_mods ---

#[test]
fn test_snapshot_returns_sorted_copy() {
    let mut reconciled = HashMap::new();
    for (i, name) in ["ModC", "ModA", "ModB"].iter().enumerate() {
        let status = match i {
            0 => ModStatus::Downloaded,
            1 => ModStatus::Staged,
            _ => ModStatus::Enabled,
        };
        reconciled.insert(
            name.to_string(),
            ReconciledMod {
                name: name.to_string(),
                status,
                source_entry: None,
                staging_entry: None,
                game_entry: None,
                register_id: String::new(),
            },
        );
    }
    let state = ModState::new(reconciled, new_repo());

    let snapshot = state.snapshot();
    assert_eq!(snapshot.len(), 3);
    assert_eq!(snapshot[0].name, "ModA");
    assert_eq!(snapshot[1].name, "ModB");
    assert_eq!(snapshot[2].name, "ModC");
}

#[test]
fn test_snapshot_is_independent_from_state() {
    let mut state = make_state(vec![("ModA", ModStatus::Downloaded)]);
    let snapshot = state.snapshot();

    state.set_staged(&make_entry("ModA", ModEntryKind::Directory));
    assert_eq!(state.get_mod("ModA").unwrap().status, ModStatus::Staged);

    // Snapshot was taken before mutation
    assert_eq!(snapshot[0].status, ModStatus::Downloaded);
}

#[test]
fn test_get_mods_returns_all_entries() {
    let state = make_state(vec![
        ("ModA", ModStatus::Downloaded),
        ("ModB", ModStatus::Staged),
    ]);

    let names: Vec<&str> = state.get_mods().map(|m| m.name.as_str()).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"ModA"));
    assert!(names.contains(&"ModB"));
}

// --- get_mod ---

#[test]
fn test_get_mod_returns_existing() {
    let state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);
    assert!(state.get_mod("SomeMod").is_some());
}

#[test]
fn test_get_mod_returns_none_for_missing() {
    let state = make_state(vec![("SomeMod", ModStatus::Downloaded)]);
    assert!(state.get_mod("NonExistent").is_none());
}

// --- multiple mods, independent transitions ---

#[test]
fn test_independent_transitions_among_multiple_mods() {
    let mut state = make_state(vec![
        ("ModA", ModStatus::Downloaded),
        ("ModB", ModStatus::Staged),
        ("ModC", ModStatus::Enabled),
    ]);

    // Advance each
    state.set_staged(&make_entry("ModA", ModEntryKind::Directory));
    state.set_enabled(&make_entry("ModB", ModEntryKind::Directory));
    state.set_disabled("ModC");

    assert_eq!(state.get_mod("ModA").unwrap().status, ModStatus::Staged);
    assert_eq!(state.get_mod("ModB").unwrap().status, ModStatus::Enabled);
    assert_eq!(state.get_mod("ModC").unwrap().status, ModStatus::Staged);

    // Remove one
    state.remove("ModB");
    assert!(state.get_mod("ModB").is_none());
    assert_eq!(state.snapshot().len(), 2);
}

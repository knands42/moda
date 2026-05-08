use moda::games::Game;
use moda::games::StardewValley;
use std::path::PathBuf;

#[test]
fn test_new_creates_instance() {
    let sv = StardewValley::new(
        PathBuf::from("/games/stardew"),
        PathBuf::from("/mods/stardew"),
        PathBuf::from("/staging/stardew"),
    );
    assert_eq!(sv.name(), "Stardew Valley");
}

#[test]
fn test_path_getters() {
    let sv = StardewValley::new(
        PathBuf::from("/games/stardew"),
        PathBuf::from("/mods/stardew"),
        PathBuf::from("/staging/stardew"),
    );
    assert_eq!(sv.game_path(), PathBuf::from("/games/stardew"));
    assert_eq!(sv.mods_path(), PathBuf::from("/mods/stardew"));
    assert_eq!(sv.staging_path(), PathBuf::from("/staging/stardew"));
}

#[test]
fn test_registry_id() {
    assert_eq!(<StardewValley as Game>::registry_id(), "stardew_valley");
}

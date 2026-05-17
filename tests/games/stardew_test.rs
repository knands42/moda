use moda::games::Game;
use moda::games::StardewValley;
use std::path::PathBuf;

#[test]
fn test_new_creates_instance() {
    // Given: a game path
    let name = StardewValley::name();
    // Then: it returns the expected name
    assert_eq!(name, "Stardew Valley");
}

#[test]
fn test_path_getters() {
    // Given: a StardewValley instance with a known path
    let sv = StardewValley::new(PathBuf::from("/games/stardew"));
    // When: game path and mod path are queried
    let game_path = sv.game_path();
    let mod_path = sv.game_mod_path();
    // Then: they return the correct paths
    assert_eq!(game_path, PathBuf::from("/games/stardew"));
    assert_eq!(mod_path, PathBuf::from("/games/stardew/Mods"));
}

#[test]
fn test_registry_id() {
    // Given: the StardewValley type
    // When: the registry ID is queried
    let id = <StardewValley as Game>::registry_id();
    // Then: it returns the correct ID string
    assert_eq!(id, "stardew_valley");
}

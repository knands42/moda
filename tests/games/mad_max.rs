use moda::games::Game;
use moda::games::MadMax;
use std::path::PathBuf;

#[test]
fn test_new_creates_instance() {
    let sv = MadMax::new(PathBuf::from("/games/mad max"));
    assert_eq!(sv.descriptor().name, "Mad Max: Fury Road");
}

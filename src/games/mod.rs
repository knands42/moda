use std::path::PathBuf;

pub trait Game {
    fn name(&self) -> &str;
    fn game_path(&self) -> PathBuf;
    fn mods_path(&self) -> PathBuf;
    fn discover_path() -> Option<PathBuf>;
    fn registry_id() -> &'static str;
}

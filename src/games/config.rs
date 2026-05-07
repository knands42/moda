use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub game_search_paths: HashMap<String, Vec<PathBuf>>
}

pub fn load_config() -> Option<Config> {
    let path = dirs::config_dir()?.join("modmanager").join("config.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&text).ok()
}
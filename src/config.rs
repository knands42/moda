use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub game_search_paths: HashMap<String, Vec<PathBuf>>,
    pub nexus_api_key: String,
}

#[allow(dead_code)]
pub fn load_config() -> Option<Config> {
    let path = dirs::config_dir()?.join("modmanager").join("config.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&text).ok()
}

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub nexus_api_key: String,
    pub mods_root_path: String,
    pub staging_root_path: String,
    pub game_search_paths: HashMap<String, Vec<PathBuf>>,
}

impl Config {
    pub fn load_config() -> Option<Config> {
        let user_home = std::env::var("HOME").ok()?;
        let path = PathBuf::from(user_home)
            .join(".config")
            .join("moda")
            .join("config.toml");

        if !path.exists() {
            log::warn!("Config not found at {}, creating default", path.display());
            Self::create_default(&path)?;
        }

        let text = std::fs::read_to_string(&path).ok()?;
        let config: Config = toml::from_str(&text).ok()?;

        log::info!("Config loaded from {}", path.display());
        Some(config)
    }

    fn create_default(path: &std::path::Path) -> Option<()> {
        let user_home = std::env::var("HOME").ok()?;

        std::fs::create_dir_all(path.parent()?).ok()?;

        let template = include_str!("../config.toml");
        let content = template.replace("$HOME", &user_home);

        std::fs::write(path, content.as_bytes()).ok()?;

        log::info!("Default config created at {}", path.display());
        Some(())
    }
}

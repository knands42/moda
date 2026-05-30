use crate::error::ModManagerError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub nexus_api_key: String,
    pub mods_root_path: String,
    pub staging_root_path: String,
    pub game_search_paths: HashMap<String, Vec<PathBuf>>,
    pub actual_config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let base_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_file_path = PathBuf::from(&base_dir)
            .join(".config")
            .join("moda")
            .join("config.toml");

        if !config_file_path.exists() {
            log::warn!(
                "Config not found at {}, creating default",
                config_file_path.display()
            );
            if let Err(e) = Self::create_default(&config_file_path) {
                log::error!("Failed to create default config: {}", e);
                return Config::load_default_config(&base_dir);
            }
        }

        Config::load_config(&config_file_path)
            .unwrap_or_else(|| Config::load_default_config(&base_dir))
    }

    fn load_default_config(base_dir: &str) -> Config {
        Config {
            nexus_api_key: String::new(),
            mods_root_path: format!("{}/.moda/mods", base_dir),
            staging_root_path: format!("{}/.moda/staging", base_dir),
            game_search_paths: HashMap::new(),
            actual_config_path: PathBuf::from(&base_dir)
                .join(".config")
                .join("moda")
                .join("config.toml"),
        }
    }
}

impl Config {
    pub fn write_new_game_path(&mut self, game_name: &str, path: PathBuf) {
        if let Some(ref mut game_paths) = self.game_search_paths.get_mut(game_name) {
            game_paths.push(path);
        } else {
            self.game_search_paths
                .insert(game_name.to_string(), vec![path]);
        }

        let new_text = match toml::to_string(self) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to serialize config: {}", e);
                return;
            }
        };

        if self.create_new_config(&new_text).is_err() {
            return;
        };

        Config::load_config(&self.actual_config_path);
    }

    fn create_new_config(&self, content: &str) -> Result<(), ModManagerError> {
        if let Err(e) = std::fs::write(&self.actual_config_path, content) {
            log::error!("Failed to write config: {}", e);
            return Err(ModManagerError::IoError(e));
        }

        log::info!("New game path {} added to config", content);
        Ok(())
    }

    fn load_config(path: &PathBuf) -> Option<Config> {
        let text = std::fs::read_to_string(path).ok()?;
        let config: Config = toml::from_str(&text).ok()?;

        log::info!("Config loaded from {}", path.display());
        Some(config)
    }

    fn create_default(path: &std::path::Path) -> Result<(), ModManagerError> {
        let user_home = std::env::var("HOME")?;

        std::fs::create_dir_all(path.parent().unwrap_or(path))?;

        let template = include_str!("../config.toml");
        let content = template.replace("$HOME", &user_home);

        std::fs::write(path, content.as_bytes())?;

        log::info!("Default config created at {}", path.display());
        Ok(())
    }
}

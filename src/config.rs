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
        let path = dirs::config_dir()?.join("moda").join("config.toml");

        if !path.exists() {
            Self::create_default(&path)?;
        }

        let text = std::fs::read_to_string(&path).ok()?;
        let config: Config = toml::from_str(&text).ok()?;

        Some(config)
    }

    fn create_default(path: &std::path::Path) -> Option<()> {
        let user_home = std::env::var("HOME").ok()?;
        let default_mods = PathBuf::from(&user_home).join("Mods");
        let default_staging = PathBuf::from(&user_home).join("Mods").join(".staging");

        std::fs::create_dir_all(path.parent()?).ok()?;

        let content = format!(
            r#"nexus_api_key = ""
mods_root_path = "{}"
staging_root_path = "{}"
"#,
            default_mods.to_string_lossy(),
            default_staging.to_string_lossy(),
        );

        std::fs::write(path, content).ok()?;
        Some(())
    }
}

use moda::config::Config;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

static HOME_LOCK: Mutex<()> = Mutex::new(());

fn with_isolated_home() -> (TempDir, String) {
    let tmp_dir = TempDir::new().unwrap();
    let home = tmp_dir.path().to_str().unwrap().to_string();
    std::env::set_var("HOME", &home);
    (tmp_dir, home)
}

#[test]
fn test_config_creates_defaults_when_no_config_file_exists() {
    // Given: a clean home directory with no existing config file
    let _guard = HOME_LOCK.lock().unwrap();
    let (_tmp_dir, home) = with_isolated_home();

    // When: Config::new() is called
    let config = Config::new();

    // Then: default values from the template are used
    assert_eq!(config.nexus_api_key, "your-key-here");
    assert_eq!(config.mods_root_path, format!("{}/.moda/mods", home));
    assert_eq!(config.staging_root_path, format!("{}/.moda/staging", home));
    assert_eq!(
        config.actual_config_path,
        PathBuf::from(&home)
            .join(".config")
            .join("moda")
            .join("config.toml")
    );
    assert!(config.game_search_paths.contains_key("stardew_valley"));
}

#[test]
fn test_config_loads_existing_valid_toml_file() {
    // Given: a pre-existing config.toml with custom values
    let _guard = HOME_LOCK.lock().unwrap();
    let (_tmp_dir, home) = with_isolated_home();

    let config_dir = PathBuf::from(&home).join(".config").join("moda");
    fs::create_dir_all(&config_dir).unwrap();

    let custom_toml = r#"nexus_api_key = "test-key-abc"
mods_root_path = "/custom/mods"
staging_root_path = "/custom/staging"
actual_config_path = "/custom/config.toml"

[game_search_paths]
skyrim = ["/games/skyrim"]
"#;
    fs::write(config_dir.join("config.toml"), custom_toml).unwrap();

    // When: Config::new() is called
    let config = Config::new();

    // Then: values are parsed from the file
    assert_eq!(config.nexus_api_key, "test-key-abc");
    assert_eq!(config.mods_root_path, "/custom/mods");
    assert_eq!(config.staging_root_path, "/custom/staging");
    assert_eq!(
        config.actual_config_path,
        PathBuf::from("/custom/config.toml")
    );
    let skyrim_paths = config.game_search_paths.get("skyrim").unwrap();
    assert_eq!(skyrim_paths.len(), 1);
    assert_eq!(skyrim_paths[0], PathBuf::from("/games/skyrim"));
}

#[test]
fn test_write_new_game_path_adds_to_existing_game_entry() {
    // Given: a config that already has paths for stardew_valley
    let _guard = HOME_LOCK.lock().unwrap();
    let (_tmp_dir, _home) = with_isolated_home();
    let mut config = Config::new();
    let original_count = config
        .game_search_paths
        .get("stardew_valley")
        .map(|v| v.len())
        .unwrap_or(0);

    // When: write_new_game_path is called for the same game
    let extra_path = PathBuf::from("/extra/stardew/path");
    config.write_new_game_path("stardew_valley", extra_path.clone());

    // Then: the new path is appended to the existing entry
    let paths = config.game_search_paths.get("stardew_valley").unwrap();
    assert_eq!(paths.len(), original_count + 1);
    assert_eq!(paths.last().unwrap(), &extra_path);
}

#[test]
fn test_write_new_game_path_creates_new_game_entry() {
    // Given: a default config with standard game entries
    let _guard = HOME_LOCK.lock().unwrap();
    let (_tmp_dir, _home) = with_isolated_home();
    let mut config = Config::new();

    // When: write_new_game_path is called for a game not yet tracked
    let path = PathBuf::from("/games/cyberpunk");
    config.write_new_game_path("cyberpunk_2077", path.clone());

    // Then: a new entry is created under that game name
    let paths = config.game_search_paths.get("cyberpunk_2077").unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], path);
}

#[test]
fn test_config_falls_back_to_defaults_on_invalid_toml() {
    // Given: a config file with unparseable content
    let _guard = HOME_LOCK.lock().unwrap();
    let (_tmp_dir, home) = with_isolated_home();

    let config_dir = PathBuf::from(&home).join(".config").join("moda");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "[[[broken toml").unwrap();

    // When: Config::new() is called
    let config = Config::new();

    // Then: it falls back to the hardcoded defaults
    assert!(config.nexus_api_key.is_empty());
    assert_eq!(config.mods_root_path, format!("{}/.moda/mods", home));
    assert_eq!(config.staging_root_path, format!("{}/.moda/staging", home));
}

#[test]
fn test_config_falls_back_to_tmp_when_home_env_is_unset() {
    // Given: HOME environment variable is not present
    let _guard = HOME_LOCK.lock().unwrap();
    let original_home = std::env::var("HOME").ok();
    std::env::remove_var("HOME");

    // When: Config::new() is called
    let config = Config::new();

    // Then: /tmp is used as the base directory
    assert!(config.nexus_api_key.is_empty());
    assert_eq!(config.mods_root_path, "/tmp/.moda/mods");
    assert_eq!(config.staging_root_path, "/tmp/.moda/staging");

    // Cleanup: restore original HOME
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

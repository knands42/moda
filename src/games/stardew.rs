use crate::error::ModManagerError;
use crate::games::{Game, ModMode};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct StardewValley {
    game_path: PathBuf,
}

impl StardewValley {
    const SMAPI_VERSION: &'static str = "4.5.2";

    pub fn new(game_path: PathBuf) -> Self {
        let stardew_valley = Self {
            game_path: game_path.clone(),
        };

        let mods_path = stardew_valley.game_mod_path();
        if !mods_path.exists() {
            fs::create_dir_all(&mods_path).ok();
            log::info!("Created Mods folder at {}", mods_path.display());
        }

        match stardew_valley.pre_setup() {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "Failed to install SMAPI for stardew valley, require manual setup: {}",
                    e
                );
            }
        }

        log::info!("Stardew Valley initialized at {}", game_path.display());
        stardew_valley
    }
}

impl Game for StardewValley {
    fn name() -> &'static str {
        "Stardew Valley"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn set_game_path(&mut self, game_path: PathBuf) {
        self.game_path = game_path;
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.join("Mods")
    }

    fn pre_setup(&self) -> Result<(), ModManagerError> {
        if self.game_path.join("SMAPI.Installer.dll").exists() {
            log::info!("SMAPI already installed");
            return Ok(());
        }

        log::info!(
            "SMAPI not found. Downloading SMAPI {}...",
            Self::SMAPI_VERSION
        );

        let url = std::env::var("MODA_SMAPI_DOWNLOAD_URL").unwrap_or_else(|_| {
            format!(
                "https://github.com/Pathoschild/SMAPI/releases/download/{v}/SMAPI-{v}-installer.zip",
                v = Self::SMAPI_VERSION
            )
        });

        let zip_data = reqwest::blocking::get(&url)
            .and_then(|r| r.bytes())
            .map_err(|e| {
                ModManagerError::GameSetupFailed(format!("Failed to download SMAPI: {e}"))
            })?;

        let cursor = io::Cursor::new(zip_data.to_vec());
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            ModManagerError::GameSetupFailed(format!("Failed to parse SMAPI archive: {e}"))
        })?;

        let temp = std::env::temp_dir().join("moda-smapi");
        let _ = fs::remove_dir_all(&temp);

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                ModManagerError::IoError(io::Error::new(io::ErrorKind::InvalidData, e))
            })?;
            let Some(path) = entry.enclosed_name() else {
                continue;
            };
            let target = temp.join(&path);
            if entry.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                fs::create_dir_all(target.parent().unwrap())?;
                let mut file = fs::File::create(&target)?;
                io::copy(&mut entry, &mut file)?;
            }
        }

        let platform_dir = find_smapi_platform_dir(&temp)?;
        copy_dir_all(&platform_dir, &self.game_path)?;
        fs::remove_dir_all(&temp)?;

        log::info!(
            "SMAPI {} installed to {}",
            Self::SMAPI_VERSION,
            self.game_path.display()
        );
        Ok(())
    }

    fn registry_id() -> &'static str {
        "stardew_valley"
    }
    
    fn mod_mode() -> ModMode { ModMode::Symlink }
}

fn find_smapi_platform_dir(extract_dir: &Path) -> Result<PathBuf, ModManagerError> {
    let internal = find_dir_containing(extract_dir, "internal").ok_or_else(|| {
        ModManagerError::InvalidConfiguration("SMAPI archive missing 'internal' directory".into())
    })?;
    let internal = internal.join("internal");

    for platform in &["Linux", "Unix", "Mono", "linux"] {
        let dir = internal.join(platform);
        if dir.is_dir() {
            return Ok(dir);
        }
    }

    Err(ModManagerError::InvalidConfiguration(
        "SMAPI archive missing Linux/Unix/Mono platform directory".into(),
    ))
}

fn find_dir_containing(root: &Path, target: &str) -> Option<PathBuf> {
    let entries: Vec<_> = fs::read_dir(root)
        .ok()?
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    for entry in &entries {
        let path = entry.path();
        if path.is_dir() {
            if entry.file_name() == target {
                return Some(root.to_path_buf());
            }
            if path.join(target).exists() {
                return Some(path);
            }
            if let Some(found) = find_dir_containing(&path, target) {
                return Some(found);
            }
        }
    }
    None
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), ModManagerError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

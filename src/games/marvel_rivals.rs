use crate::error::ModManagerError;
use crate::games::{Game, ModMode};
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct MarvelRivals {
    game_path: PathBuf,
}

impl MarvelRivals {
    pub fn new(game_path: PathBuf) -> Self {
        let marvel_rivals = Self {
            game_path: game_path.clone(),
        };

        let mods_path = marvel_rivals.game_mod_path();
        if !mods_path.exists() {
            fs::create_dir_all(&mods_path).ok();
        }

        match marvel_rivals.pre_setup() {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "Failed to install UTOC signature bypass for Marvel Rivals, require manual setup: {}",
                    e
                );
            }
        }

        log::info!("Marvel Rivals initialized at {}", game_path.display());
        marvel_rivals
    }
}

impl Game for MarvelRivals {
    fn name() -> &'static str {
        "Marvel Rivals"
    }

    fn game_path(&self) -> PathBuf {
        self.game_path.clone()
    }

    fn set_game_path(&mut self, game_path: PathBuf) {
        self.game_path = game_path;
    }

    fn game_mod_path(&self) -> PathBuf {
        self.game_path.join("MarvelGame/Marvel/Content/Paks/~mods")
    }

    fn pre_setup(&self) -> Result<(), ModManagerError> {
        let target_dir = self.game_path.join("MarvelGame/Marvel/Binaries/Win64");

        if target_dir.join("plugins/MarvelRivalsUTOCSignatureBypass.asi").exists() {
            log::info!("UTOC signature bypass already installed");
            return Ok(());
        }

        log::info!("UTOC signature bypass not found. Downloading...");

        let url = std::env::var("MODA_MARVEL_RIVALS_UTOC_URL").unwrap_or_else(|_| {
            "https://github.com/DeathChaos25/MarvelRivalsUTOCSignatureBypass/releases/download/1.0.0/Marvel.Rivals.UTOC.Signature.Bypass.Patch.zip".into()
        });

        let zip_data = reqwest::blocking::get(&url)
            .and_then(|r| r.bytes())
            .map_err(|e| {
                ModManagerError::GameSetupFailed(format!(
                    "Failed to download UTOC signature bypass: {e}"
                ))
            })?;

        let cursor = io::Cursor::new(zip_data.to_vec());
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            ModManagerError::GameSetupFailed(format!(
                "Failed to parse UTOC signature bypass archive: {e}"
            ))
        })?;

        let temp = std::env::temp_dir().join("moda-marvel-rivals-utoc");
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

        fs::create_dir_all(&target_dir)?;
        for entry in fs::read_dir(&temp)? {
            let entry = entry?;
            let src = entry.path();
            let dst = target_dir.join(entry.file_name());
            if src.is_dir() {
                copy_dir_all(&src, &dst)?;
            } else {
                fs::copy(&src, &dst)?;
            }
        }
        fs::remove_dir_all(&temp)?;

        log::info!(
            "UTOC signature bypass installed to {}",
            target_dir.display()
        );
        Ok(())
    }

    fn registry_id() -> &'static str {
        "marvel_rivals"
    }
    
    fn mod_mode() -> ModMode { ModMode::Symlink }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<(), ModManagerError> {
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

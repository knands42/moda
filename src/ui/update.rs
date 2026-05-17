use std::path::PathBuf;

use crate::config::Config;
use crate::games::{Game, StardewValley};
use crate::mods::SyncManager;

use super::app::App;
use super::message::{Message, Tab};

pub fn update(app: &mut App, message: Message) {
    match message {
        Message::TabSelected(tab) => {
            log::debug!("Tab switched to {tab:?}");
            app.current_tab = tab;
        }

        Message::SelectGame(id) => {
            log::info!("Game selected: {id}");
            if id == StardewValley::registry_id() {
                let config = match Config::load_config() {
                    Some(c) => c,
                    None => {
                        app.push_log("Failed to load config".to_string());
                        return;
                    }
                };

                let game_path = match StardewValley::discover_path(&config) {
                    Some(p) => p,
                    None => {
                        app.push_log("Stardew Valley not found".to_string());
                        // TODO: pop-up to manually input
                        return;
                    }
                };

                let mods = PathBuf::from(&config.mods_root_path).join(StardewValley::registry_id());
                let staging =
                    PathBuf::from(&config.staging_root_path).join(StardewValley::registry_id());

                app.game_name = StardewValley::name().to_string();
                app.game_path = game_path.to_string_lossy().to_string();
                app.mods_path = mods.to_string_lossy().to_string();
                app.staging_path = staging.to_string_lossy().to_string();
                app.game_mod_path = game_path.join("Mods").to_string_lossy().to_string();
                app.sync_manager = Some(Box::new(SyncManager::new(
                    StardewValley::new(game_path),
                    config,
                )));
                app.current_tab = Tab::Mods;
                app.push_log(format!("Selected game: {}", StardewValley::name()));
            }
        }

        Message::Reconcile => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Reconcile requested");
            let path = PathBuf::from(&app.game_mod_path);
            match sm.reconcile(&path) {
                Ok(state) => {
                    app.mod_state = state;
                    app.push_log("Reconciled mods");
                    log::info!("Reconcile completed successfully");
                }
                Err(e) => {
                    log::error!("Reconcile failed: {e}");
                    app.push_log(format!("Reconcile failed: {e}"));
                }
            }
        }

        Message::SyncAll => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Sync all requested");
            match sm.sync_all(&mut app.mod_state) {
                Ok(()) => {
                    app.push_log("Sync completed");
                    log::info!("Sync all completed");
                }
                Err(e) => {
                    log::error!("Sync all failed: {e}");
                    app.push_log(format!("Sync failed: {e}"));
                }
            }
        }

        Message::StageMod(name) => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Stage mod requested: {name}");
            let entry = app
                .mod_state
                .get_mod(&name)
                .and_then(|m| m.source_entry.clone());
            match entry {
                Some(e) => match sm.stage_one_mod(&e, &mut app.mod_state) {
                    Ok(()) => app.push_log(format!("Staged {name}")),
                    Err(e) => {
                        log::error!("Stage failed for {name}: {e}");
                        app.push_log(format!("Stage failed: {e}"));
                    }
                },
                None => app.push_log(format!("{name} not found in downloads")),
            }
        }

        Message::UnstageMod(name) => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Unstage mod requested: {name}");
            let entry = app
                .mod_state
                .get_mod(&name)
                .and_then(|m| m.staging_entry.clone());
            match entry {
                Some(e) => match sm.unstage_one_mod(&e, &mut app.mod_state) {
                    Ok(()) => app.push_log(format!("Unstaged {name}")),
                    Err(e) => {
                        log::error!("Unstage failed for {name}: {e}");
                        app.push_log(format!("Unstage failed: {e}"));
                    }
                },
                None => app.push_log(format!("{name} not found in staging")),
            }
        }

        Message::EnableMod(name) => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Enable mod requested: {name}");
            let entry = app
                .mod_state
                .get_mod(&name)
                .and_then(|m| m.staging_entry.clone());
            match entry {
                Some(e) => match sm.enable_one_mod(&e, &mut app.mod_state) {
                    Ok(()) => app.push_log(format!("Enabled {name}")),
                    Err(e) => {
                        log::error!("Enable failed for {name}: {e}");
                        app.push_log(format!("Enable failed: {e}"));
                    }
                },
                None => app.push_log(format!("{name} not found in staging")),
            }
        }

        Message::DisableMod(name) => {
            let sm = match app.sync_manager.as_ref() {
                Some(sm) => sm,
                None => return,
            };
            log::info!("Disable mod requested: {name}");
            let entry = app
                .mod_state
                .get_mod(&name)
                .and_then(|m| m.game_entry.clone());
            match entry {
                Some(e) => match sm.disable_one_mod(&e, &mut app.mod_state) {
                    Ok(()) => app.push_log(format!("Disabled {name}")),
                    Err(e) => {
                        log::error!("Disable failed for {name}: {e}");
                        app.push_log(format!("Disable failed: {e}"));
                    }
                },
                None => app.push_log(format!("{name} not found in game mods")),
            }
        }
    }
}

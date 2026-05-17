use std::path::PathBuf;

use crate::config::Config;
use crate::games::{Game, StardewValley};
use crate::mods::SyncManager;

use super::app::App;
use super::message::Message;

pub fn update(app: &mut App, message: Message) {
    match message {
        Message::TabSelected(tab) => {
            log::debug!("Tab switched to {tab:?}");
            app.current_tab = tab;
        }

        Message::Reconcile => with_sync_manager(app, |app, sm| {
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
        }),

        Message::SyncAll => {
            log::info!("Sync all requested");
            with_sync_manager(app, |app, sm| match sm.sync_all(&mut app.mod_state) {
                Ok(()) => {
                    app.push_log("Sync completed");
                    log::info!("Sync all completed");
                }
                Err(e) => {
                    log::error!("Sync all failed: {e}");
                    app.push_log(format!("Sync failed: {e}"));
                }
            })
        }

        Message::StageMod(name) => {
            log::info!("Stage mod requested: {name}");
            with_sync_manager(app, |app, sm| {
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
            })
        }

        Message::UnstageMod(name) => {
            log::info!("Unstage mod requested: {name}");
            with_sync_manager(app, |app, sm| {
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
            })
        }

        Message::EnableMod(name) => {
            log::info!("Enable mod requested: {name}");
            with_sync_manager(app, |app, sm| {
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
            })
        }

        Message::DisableMod(name) => {
            log::info!("Disable mod requested: {name}");
            with_sync_manager(app, |app, sm| {
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
            })
        }
    }
}

fn with_sync_manager<F>(app: &mut App, f: F)
where
    F: FnOnce(&mut App, SyncManager<StardewValley>),
{
    match make_sync_manager() {
        Ok(sm) => f(app, sm),
        Err(e) => app.push_log(format!("Init failed: {e}")),
    }
}

fn make_sync_manager() -> Result<SyncManager<StardewValley>, String> {
    let config = Config::load_config().ok_or("Failed to load config")?;
    let game_path = StardewValley::discover_path(&config).ok_or("Stardew Valley not found")?;
    let game = StardewValley::new(game_path);
    Ok(SyncManager::new(game, config))
}

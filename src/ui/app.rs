use crate::config::Config;
use crate::games::{Game, MarvelRivals, StardewValley};
use crate::mods::SyncManager;
use crate::ui::active_game::ActiveGame;

use super::style;
use super::widgets::dir_browser::DirBrowser;

use std::path::PathBuf;

pub enum Tab {
    Downloads,
    Staging,
}

pub struct PathDialogState {
    pub game_id: String,
    pub path: String,
}

pub struct ModaApp {
    config: Config,
    page: Page,
    error: Option<String>,
    path_dialog: Option<PathDialogState>,
    dir_browser: DirBrowser,
    active_game: Option<ActiveGame>,
    active_tab: Tab,
    pending_select: Option<PathBuf>,
}

enum Page {
    GameSelection,
    ModManager,
}

impl Default for ModaApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ModaApp {
    pub fn new() -> Self {
        let config = Config::load_config().unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            log::warn!("Failed to load config, using fallback defaults");
            Config {
                nexus_api_key: String::new(),
                mods_root_path: format!("{}/Mods", home),
                staging_root_path: format!("{}/Mods/.staging", home),
                game_search_paths: std::collections::HashMap::new(),
            }
        });

        Self {
            config,
            page: Page::GameSelection,
            error: None,
            path_dialog: None,
            dir_browser: DirBrowser::new(),
            active_game: None,
            active_tab: Tab::Downloads,
            pending_select: None,
        }
    }

    fn create_game_and_switch(&mut self, game_id: &str, path: PathBuf) {
        let result = match game_id {
            "stardew_valley" => {
                let game = StardewValley::new(path);
                let sm = SyncManager::new(game, self.config.clone());
                sm.reconcile(&sm.game_mod_path())
                    .map(|state| ActiveGame::StardewValley(sm, state))
            }
            "marvel_rivals" => {
                let game = MarvelRivals::new(path);
                let sm = SyncManager::new(game, self.config.clone());
                sm.reconcile(&sm.game_mod_path())
                    .map(|state| ActiveGame::MarvelRivals(sm, state))
            }
            _ => unreachable!(),
        };

        match result {
            Ok(active) => {
                self.active_game = Some(active);
                self.page = Page::ModManager;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to initialize game: {}", e));
            }
        }
    }
}

impl eframe::App for ModaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(style::configure_visuals());

        // --- Top bar ---
        egui::Panel::top("top_bar")
            .min_size(0.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let is_manager = matches!(self.page, Page::ModManager);

                    if is_manager {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("\u{2190}  Back")
                                        .size(13.0)
                                        .color(style::ACCENT),
                                )
                                .fill(style::SURFACE)
                                .min_size(egui::vec2(60.0, 24.0)),
                            )
                            .clicked()
                        {
                            self.page = Page::GameSelection;
                            self.active_game = None;
                            self.active_tab = Tab::Downloads;
                        }
                        ui.add_space(8.0);
                    }

                    ui.label(
                        egui::RichText::new("Moda")
                            .size(16.0)
                            .color(style::ACCENT)
                            .strong(),
                    );

                    if is_manager {
                        if let Some(ref game) = self.active_game {
                            ui.label(
                                egui::RichText::new(format!("/ {}", game.game_name()))
                                    .size(14.0)
                                    .color(style::TEXT_MUTED),
                            );
                        }
                    }
                });
                ui.add_space(8.0);
            });

        // --- Central content ---
        egui::CentralPanel::default().show_inside(ui, |ui| match self.page {
            Page::GameSelection => {
                let game = crate::ui::pages::game_selection::render(
                    ui,
                    &self.config,
                    &mut self.path_dialog,
                    &mut self.error,
                );
                if let Some(active_game) = game {
                    self.active_game = Some(active_game);
                    self.page = Page::ModManager;
                }
            }
            Page::ModManager => {
                if let Some(ref mut active_game) = self.active_game {
                    crate::ui::pages::mod_manager::render(
                        ui,
                        active_game,
                        &mut self.active_tab,
                        &mut self.error,
                    );
                }
            }
        });

        let ctx = ui.ctx();

        // --- Path dialog ---
        if self.path_dialog.is_some() {
            let game_id = self.path_dialog.as_ref().unwrap().game_id.clone();
            let game_name = match game_id.as_str() {
                "stardew_valley" => StardewValley::name(),
                "marvel_rivals" => MarvelRivals::name(),
                _ => "Unknown",
            };

            let mut close = false;
            let mut confirm = false;
            let mut browse = false;

            egui::Window::new("Game Path Required")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .default_size([450.0, 200.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Could not find {}. Please locate the installation directory.",
                                game_name
                            ))
                            .size(14.0),
                        );
                        ui.add_space(12.0);

                        let path = &mut self.path_dialog.as_mut().unwrap().path;
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Path:")
                                    .size(13.0)
                                    .color(style::TEXT_MUTED),
                            );
                            ui.add(
                                egui::TextEdit::singleline(path)
                                    .hint_text("/path/to/game")
                                    .desired_width(f32::INFINITY),
                            );
                            if ui
                                .button(
                                    egui::RichText::new("Browse...")
                                        .size(13.0)
                                        .color(style::ACCENT),
                                )
                                .clicked()
                            {
                                browse = true;
                            }
                        });

                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            if ui
                                .button(egui::RichText::new("Cancel").size(13.0))
                                .clicked()
                            {
                                close = true;
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Confirm")
                                                    .size(13.0)
                                                    .color(egui::Color32::WHITE),
                                            )
                                            .fill(style::ACCENT)
                                            .min_size(egui::vec2(100.0, 28.0)),
                                        )
                                        .clicked()
                                    {
                                        confirm = true;
                                    }
                                },
                            );
                        });
                    });
                });

            if browse {
                self.dir_browser.visible = true;
            }

            if confirm {
                let path = self.path_dialog.as_ref().unwrap().path.clone();
                if PathBuf::from(&path).exists() && PathBuf::from(&path).is_dir() {
                    self.create_game_and_switch(&game_id, PathBuf::from(&path));
                    close = true;
                } else {
                    self.error = Some("Path does not exist or is not a directory".to_string());
                }
            }

            if close {
                self.path_dialog = None;
            }
        }

        // --- Directory browser modal ---
        if self.dir_browser.visible {
            self.dir_browser.show(ctx, &mut self.pending_select);
            if let Some(selected) = self.pending_select.take() {
                if let Some(ref mut state) = self.path_dialog {
                    state.path = selected.to_string_lossy().to_string();
                }
            }
        }

        // --- Error display ---
        if let Some(ref err) = self.error.clone() {
            let mut dismiss = false;
            egui::Window::new("Error")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .default_size([400.0, 120.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(err).color(style::ERROR).size(14.0));
                    ui.add_space(12.0);
                    if ui
                        .add(egui::Button::new("Dismiss").min_size(egui::vec2(80.0, 28.0)))
                        .clicked()
                    {
                        dismiss = true;
                    }
                });
            if dismiss {
                self.error = None;
            }
        }
    }
}

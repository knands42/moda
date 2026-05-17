use super::super::style;
use crate::config::Config;
use crate::games::{self, Game, MarvelRivals, StardewValley};
use crate::mods::SyncManager;
use crate::ui::active_game::ActiveGame;

pub fn render(
    ui: &mut egui::Ui,
    config: &Config,
    path_dialog: &mut Option<super::super::app::PathDialogState>,
    error: &mut Option<String>,
) -> Option<ActiveGame> {
    let mut result: Option<ActiveGame> = None;

    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        ui.label(
            egui::RichText::new("Moda")
                .size(36.0)
                .color(style::ACCENT)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Select a Game")
                .size(16.0)
                .color(style::TEXT_MUTED),
        );
        ui.add_space(32.0);
    });

    let games_list = games::supported_games();
    let container_width = ui.available_width() - 48.0;
    let card_width = (container_width / games_list.len().max(1) as f32).min(320.0);

    ui.horizontal_centered(|ui| {
        ui.add_space((ui.available_width() - card_width * games_list.len() as f32) / 2.0);

        for (id, name, description) in &games_list {
            let (rect, card_response) =
                ui.allocate_exact_size(egui::vec2(card_width - 12.0, 200.0), egui::Sense::click());

            if ui.is_rect_visible(rect) {
                let is_hovered = card_response.hovered();
                let bg = if is_hovered {
                    style::CARD_HOVER
                } else {
                    style::CARD_BG
                };

                ui.painter().rect_filled(rect, 12.0, bg);
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::same(12),
                    egui::Stroke::new(1.0, style::BORDER),
                    egui::StrokeKind::Outside,
                );

                if is_hovered {
                    ui.painter().rect_stroke(
                        rect,
                        egui::CornerRadius::same(12),
                        egui::Stroke::new(1.5, style::ACCENT),
                        egui::StrokeKind::Outside,
                    );
                }

                let icon = icon_for_game(name);
                ui.painter().text(
                    egui::Pos2::new(rect.center().x, rect.top() + 40.0),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(36.0),
                    style::TEXT,
                );

                ui.painter().text(
                    egui::Pos2::new(rect.center().x, rect.top() + 80.0),
                    egui::Align2::CENTER_CENTER,
                    *name,
                    egui::FontId::proportional(18.0),
                    style::TEXT,
                );

                ui.painter().text(
                    egui::Pos2::new(rect.center().x, rect.top() + 140.0),
                    egui::Align2::CENTER_CENTER,
                    *description,
                    egui::FontId::proportional(12.0),
                    style::TEXT_MUTED,
                );
            }

            if card_response.clicked() {
                let registry_id: &str = id;
                if registry_id == "stardew_valley" {
                    if let Some(path) = StardewValley::discover_path(config) {
                        let game = StardewValley::new(path);
                        let sm = SyncManager::new(game, config.clone());
                        match sm.reconcile(&sm.game_mod_path()) {
                            Ok(state) => {
                                result = Some(ActiveGame::StardewValley(sm, state));
                            }
                            Err(e) => {
                                *error = Some(format!("Reconcile failed: {}", e));
                            }
                        }
                    } else {
                        *path_dialog = Some(super::super::app::PathDialogState {
                            game_id: "stardew_valley".to_string(),
                            path: String::new(),
                        });
                    }
                } else if registry_id == "marvel_rivals" {
                    if let Some(path) = MarvelRivals::discover_path(config) {
                        let game = MarvelRivals::new(path);
                        let sm = SyncManager::new(game, config.clone());
                        match sm.reconcile(&sm.game_mod_path()) {
                            Ok(state) => {
                                result = Some(ActiveGame::MarvelRivals(sm, state));
                            }
                            Err(e) => {
                                *error = Some(format!("Reconcile failed: {}", e));
                            }
                        }
                    } else {
                        *path_dialog = Some(super::super::app::PathDialogState {
                            game_id: "marvel_rivals".to_string(),
                            path: String::new(),
                        });
                    }
                }
            }

            ui.add_space(12.0);
        }
    });

    ui.add_space(16.0);

    if let Some(ref err) = error {
        ui.label(
            egui::RichText::new(format!("Error: {}", err))
                .color(style::ERROR)
                .size(13.0),
        );
    }

    result
}

fn icon_for_game(name: &str) -> &str {
    match name {
        "Stardew Valley" => "\u{1F33E}",
        "Marvel Rivals" => "\u{2694}\u{FE0F}",
        _ => "\u{1F3AE}",
    }
}

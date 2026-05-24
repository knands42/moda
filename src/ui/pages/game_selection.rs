use super::super::app::PathDialogState;
use super::super::components::game_card;
use super::super::style;
use crate::config::Config;
use crate::games;
use crate::ui::active_game::ActiveGame;

pub fn render(
    ui: &mut egui::Ui,
    config: &Config,
    path_dialog: &mut Option<PathDialogState>,
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

    let games_list = games::registered_games();
    let container_width = ui.available_width() - 48.0;
    let card_width = (container_width / games_list.len().max(1) as f32).min(320.0);

    ui.horizontal_centered(|ui| {
        ui.add_space((ui.available_width() - card_width * games_list.len() as f32) / 2.0);

        for &descriptor in games_list {
            let (rect, card_response) =
                ui.allocate_exact_size(egui::vec2(card_width - 12.0, 200.0), egui::Sense::click());

            if ui.is_rect_visible(rect) {
                let is_hovered = card_response.hovered();
                game_card::render(
                    ui,
                    rect,
                    is_hovered,
                    descriptor.icon,
                    descriptor.name,
                    descriptor.description,
                );
            }

            if card_response.clicked() {
                if let Some(path) = descriptor.discover_path(config) {
                    match ActiveGame::create(descriptor, path, config) {
                        Ok(active_game) => result = Some(active_game),
                        Err(err) => *error = Some(format!("Reconcile failed: {}", err)),
                    }
                } else {
                    *path_dialog = Some(PathDialogState {
                        descriptor,
                        path: String::new(),
                    });
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

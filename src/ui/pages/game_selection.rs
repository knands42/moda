use std::path::PathBuf;

use super::super::app::PathDialogState;
use super::super::components::game_card;
use super::super::style;
use crate::config::Config;
use crate::error::ModManagerError;
use crate::games::{self, Game, MarvelRivals, StardewValley};
use crate::mods::{ModState, SyncManager};
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

            let entry = get_game_info(id);

            if ui.is_rect_visible(rect) {
                let is_hovered = card_response.hovered();
                let icon = entry.map(|e| e.icon).unwrap_or("\u{1F3AE}");
                game_card::render(ui, rect, is_hovered, icon, name, description);
            }

            if card_response.clicked() {
                if let Some(entry) = entry {
                    match (entry.construct)(config) {
                        Ok(active_game) => result = Some(active_game),
                        Err(SelectionError::NeedsPath(state)) => *path_dialog = Some(state),
                        Err(SelectionError::ReconcileFailed(err)) => {
                            *error = Some(format!("Reconcile failed: {}", err));
                        }
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

#[derive(Copy, Clone)]
struct GameEntry {
    icon: &'static str,
    construct: fn(&Config) -> Result<ActiveGame, SelectionError>,
}

fn get_game_info(registry_id: &str) -> Option<GameEntry> {
    match registry_id {
        "stardew_valley" => Some(GameEntry {
            icon: "\u{1F33E}",
            construct: |c| {
                select_game::<StardewValley>(c, StardewValley::new, ActiveGame::StardewValley)
            },
        }),
        "marvel_rivals" => Some(GameEntry {
            icon: "\u{2694}\u{FE0F}",
            construct: |c| {
                select_game::<MarvelRivals>(c, MarvelRivals::new, ActiveGame::MarvelRivals)
            },
        }),
        _ => None,
    }
}

enum SelectionError {
    NeedsPath(PathDialogState),
    ReconcileFailed(ModManagerError),
}

fn select_game<G: Game + 'static>(
    config: &Config,
    construct: fn(PathBuf) -> G,
    make_active: fn(SyncManager<G>, ModState) -> ActiveGame,
) -> Result<ActiveGame, SelectionError> {
    if let Some(path) = G::discover_path(config) {
        let game = construct(path);
        let sm = SyncManager::new(game, config.clone());
        sm.reconcile(&sm.game_mod_path())
            .map(|state| make_active(sm, state))
            .map_err(SelectionError::ReconcileFailed)
    } else {
        Err(SelectionError::NeedsPath(PathDialogState {
            game_name: G::name(),
            path: String::new(),
            creator: Box::new(move |config, path| {
                let game = construct(path);
                let sm = SyncManager::new(game, config.clone());
                sm.reconcile(&sm.game_mod_path())
                    .map(|state| make_active(sm, state))
            }),
        }))
    }
}

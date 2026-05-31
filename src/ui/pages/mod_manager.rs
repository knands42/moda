use crate::games::ModMode;
use crate::mods::types::ModEntry;
use crate::mods::types::ModStatus;
use crate::mods::ModEntryKind;
use crate::ui::active_game::ActiveGame;

use super::super::style;

pub fn render(
    ui: &mut egui::Ui,
    active_game: &mut ActiveGame,
    active_tab: &mut super::super::app::Tab,
    error: &mut Option<String>,
) {
    let game_name = active_game.game_name();

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(game_name)
                .size(22.0)
                .color(style::ACCENT)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let reconcile_btn = egui::Button::new(
                egui::RichText::new("\u{1F503}  Reconcile")
                    .size(14.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(style::CARD_BG)
            .min_size(egui::vec2(130.0, 32.0));
            if ui.add(reconcile_btn).clicked() {
                log::info!("Reconcile triggered for {}", game_name);
                if let Err(e) = active_game.reconcile() {
                    *error = Some(format!("Reconcile failed: {}", e));
                }
            }

            let sync_btn = egui::Button::new(
                egui::RichText::new("\u{1F504}  Sync All")
                    .size(14.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(style::ACCENT)
            .min_size(egui::vec2(130.0, 32.0));
            if ui.add(sync_btn).clicked() {
                log::info!("Sync all triggered for {}", game_name);
                if let Err(e) = active_game.sync_all() {
                    *error = Some(format!("Sync failed: {}", e));
                }
                if let Err(e) = active_game.reconcile() {
                    *error = Some(format!("Re-reconcile failed: {}", e));
                }
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let downloads_selected = matches!(active_tab, super::super::app::Tab::Downloads);
        let staging_selected = matches!(active_tab, super::super::app::Tab::Staging);

        let tab_h = 32.0;
        let downloads_resp = ui.add(
            egui::Button::selectable(downloads_selected, "Downloads")
                .min_size(egui::vec2(120.0, tab_h)),
        );
        if downloads_resp.clicked() {
            *active_tab = super::super::app::Tab::Downloads;
        }

        let staging_resp = ui.add(
            egui::Button::selectable(staging_selected, "Staging")
                .min_size(egui::vec2(120.0, tab_h)),
        );
        if staging_resp.clicked() {
            *active_tab = super::super::app::Tab::Staging;
        }
    });

    ui.add_space(8.0);

    if let Some(ref err) = error {
        ui.label(
            egui::RichText::new(format!("Error: {}", err))
                .color(style::ERROR)
                .size(13.0),
        );
        ui.add_space(4.0);
    }

    match active_tab {
        super::super::app::Tab::Downloads => {
            render_downloads_tab(ui, active_game, error);
        }
        super::super::app::Tab::Staging => {
            render_staging_tab(ui, active_game, error);
        }
    }
}

fn render_downloads_tab(
    ui: &mut egui::Ui,
    active_game: &mut ActiveGame,
    error: &mut Option<String>,
) {
    let mods: Vec<_> = {
        let state = active_game.state();
        state
            .snapshot()
            .into_iter()
            .filter(|m| {
                m.status == ModStatus::Downloaded
                    || (m.status == ModStatus::Modified && m.source_entry.is_some())
            })
            .collect()
    };

    if mods.is_empty() {
        ui.add_space(32.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No mods in downloads folder")
                    .size(15.0)
                    .color(style::TEXT_MUTED),
            );
            ui.label(
                egui::RichText::new("Place mod .zip files or folders in the downloads directory")
                    .size(12.0)
                    .color(style::TEXT_MUTED),
            );
        });
        return;
    }

    let mut to_stage: Option<ModEntry> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::NONE.corner_radius(8.0).show(ui, |ui| {
            egui::Grid::new("downloads_grid")
                .striped(true)
                .spacing([12.0, 0.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Type");
                    ui.strong("");
                    ui.end_row();

                    for m in &mods {
                        if let Some(source) = &m.source_entry {
                            ui.label(egui::RichText::new(&m.name).size(13.0).color(style::TEXT));
                            ui.label(
                                egui::RichText::new(match source.kind {
                                    ModEntryKind::ZipArchive => "ZIP",
                                    ModEntryKind::RarArchive => "RAR",
                                    ModEntryKind::Directory => "Folder",
                                    ModEntryKind::PakArchive => "PAK",
                                    ModEntryKind::Other => "Other",
                                })
                                .size(12.0)
                                .color(style::TEXT_MUTED),
                            );

                            let stage_btn = egui::Button::new(
                                egui::RichText::new("Stage")
                                    .size(12.0)
                                    .color(style::SUCCESS),
                            )
                            .fill(style::CARD_BG)
                            .min_size(egui::vec2(60.0, 24.0));
                            if ui.add(stage_btn).clicked() {
                                to_stage = Some(source.clone());
                            }

                            ui.end_row();
                        }
                    }
                });
        });
    });

    if let Some(entry) = to_stage {
        if let Err(e) = active_game.stage_one_mod(&entry) {
            *error = Some(format!("Stage failed: {}", e));
        } else {
            active_game.reconcile().unwrap_or_else(|e| {
                *error = Some(format!("Re-reconcile failed: {}", e));
            });
        }
    }
}

fn render_staging_tab(ui: &mut egui::Ui, active_game: &mut ActiveGame, error: &mut Option<String>) {
    let mods: Vec<_> = {
        let state = active_game.state();
        state
            .snapshot()
            .into_iter()
            .filter(|m| {
                m.status == ModStatus::Staged
                    || m.status == ModStatus::Enabled
                    || m.status == ModStatus::Modified
            })
            .collect()
    };

    if mods.is_empty() {
        ui.add_space(32.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No staged mods")
                    .size(15.0)
                    .color(style::TEXT_MUTED),
            );
            ui.label(
                egui::RichText::new("Stage mods from the Downloads tab first")
                    .size(12.0)
                    .color(style::TEXT_MUTED),
            );
        });
        return;
    }

    let mod_mode = active_game.mod_mode();
    let mode_label = match mod_mode {
        ModMode::Symlink => "Symlink",
        ModMode::Pak => "Pak",
        ModMode::DirectCopy => "Direct Copy",
    };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Mode:")
                .size(12.0)
                .color(style::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(mode_label)
                .size(12.0)
                .color(style::ACCENT)
                .strong(),
        );
    });
    ui.add_space(6.0);

    let mut to_enable: Option<ModEntry> = None;
    let mut to_disable: Option<ModEntry> = None;
    let mut to_unstage: Option<ModEntry> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::NONE.corner_radius(8.0).show(ui, |ui| {
            egui::Grid::new("staging_grid")
                .striped(true)
                .spacing([8.0, 0.0])
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Status");
                    ui.strong("Actions");
                    ui.end_row();

                    for m in &mods {
                        let status_color = match m.status {
                            ModStatus::Enabled => style::SUCCESS,
                            ModStatus::Staged => style::WARNING,
                            ModStatus::Modified => style::ERROR,
                            _ => style::TEXT_MUTED,
                        };

                        ui.label(egui::RichText::new(&m.name).size(13.0).color(style::TEXT));

                        let status_text = match m.status {
                            ModStatus::Enabled => "\u{2705} Enabled",
                            ModStatus::Staged => "\u{23F3} Staged",
                            ModStatus::Modified => "\u{26A0}\u{FE0F} Modified",
                            _ => "Unknown",
                        };
                        ui.label(
                            egui::RichText::new(status_text)
                                .size(12.0)
                                .color(status_color),
                        );

                        ui.horizontal(|ui| {
                            if m.status == ModStatus::Staged {
                                if let Some(ref entry) = m.staging_entry {
                                    let enable_btn = egui::Button::new(
                                        egui::RichText::new("Enable")
                                            .size(12.0)
                                            .color(style::SUCCESS),
                                    )
                                    .fill(style::CARD_BG)
                                    .min_size(egui::vec2(60.0, 24.0));
                                    if ui.add(enable_btn).clicked() {
                                        to_enable = Some(entry.clone());
                                    }
                                }
                            }

                            if m.status == ModStatus::Enabled || m.status == ModStatus::Modified {
                                if let Some(ref entry) = m.game_entry {
                                    let disable_btn = egui::Button::new(
                                        egui::RichText::new("Disable")
                                            .size(12.0)
                                            .color(style::WARNING),
                                    )
                                    .fill(style::CARD_BG)
                                    .min_size(egui::vec2(60.0, 24.0));
                                    if ui.add(disable_btn).clicked() {
                                        to_disable = Some(entry.clone());
                                    }
                                }
                            }

                            let unstage_entry =
                                m.staging_entry.as_ref().or(m.game_entry.as_ref()).cloned();
                            if let Some(entry) = unstage_entry {
                                let unstage_btn = egui::Button::new(
                                    egui::RichText::new("Unstage")
                                        .size(12.0)
                                        .color(style::ERROR),
                                )
                                .fill(style::CARD_BG)
                                .min_size(egui::vec2(60.0, 24.0));
                                if ui.add(unstage_btn).clicked() {
                                    to_unstage = Some(entry);
                                }
                            }
                        });

                        ui.end_row();
                    }
                });
        });
    });

    if let Some(entry) = to_enable {
        if let Err(e) = active_game.enable_one_mod(&entry) {
            *error = Some(format!("Enable failed: {}", e));
        } else {
            active_game.reconcile().unwrap_or_else(|e| {
                *error = Some(format!("Re-reconcile failed: {}", e));
            });
        }
    }

    if let Some(entry) = to_disable {
        if let Err(e) = active_game.disable_one_mod(&entry) {
            *error = Some(format!("Disable failed: {}", e));
        } else {
            active_game.reconcile().unwrap_or_else(|e| {
                *error = Some(format!("Re-reconcile failed: {}", e));
            });
        }
    }

    if let Some(entry) = to_unstage {
        if let Err(e) = active_game.unstage_one_mod(&entry) {
            *error = Some(format!("Unstage failed: {}", e));
        } else {
            active_game.reconcile().unwrap_or_else(|e| {
                *error = Some(format!("Re-reconcile failed: {}", e));
            });
        }
    }
}

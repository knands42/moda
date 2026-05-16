use iced::widget::{button, column, row, scrollable, text, Column};
use iced::Element;
use moda::config::load_config;
use moda::games::{Game, StardewValley};
use moda::mods::mod_registry::ModStatus;
use moda::mods::{ModState, SyncManager};
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::run("Moda", update, view)
}

#[derive(Debug, Clone)]
enum Message {
    Reconcile,
    StageMod(String),
    EnableMod(String),
    StageAll,
    EnableAll,
    SyncAll,
}

struct App {
    game_path: String,
    mods_path: String,
    staging_path: String,
    game_mod_path: String,
    mod_state: ModState,
    log: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        let (game_path, mods_path, staging_path, game_mod_path) = match load_config() {
            Some(config) => {
                let discovered = StardewValley::discover_path(&config);
                match discovered {
                    Some(gp) => {
                        let mods = PathBuf::from(&config.mods_root_path)
                            .join(StardewValley::registry_id());
                        let staging = PathBuf::from(&config.staging_root_path)
                            .join(StardewValley::registry_id());
                        (
                            gp.to_string_lossy().to_string(),
                            mods.to_string_lossy().to_string(),
                            staging.to_string_lossy().to_string(),
                            gp.join("Mods").to_string_lossy().to_string(),
                        )
                    }
                    None => (
                        "Not found".into(),
                        config.mods_root_path.clone(),
                        config.staging_root_path.clone(),
                        "N/A".into(),
                    ),
                }
            }
            None => (
                "Config not loaded".into(),
                String::new(),
                String::new(),
                String::new(),
            ),
        };

        App {
            game_path,
            mods_path,
            staging_path,
            game_mod_path,
            mod_state: ModState::default(),
            log: vec!["App started. Click Reconcile to load mods.".into()],
        }
    }
}

impl App {
    fn push_log(&mut self, msg: String) {
        self.log.push(msg);
    }

    fn sync_manager(&self) -> Result<SyncManager<StardewValley>, String> {
        let config = load_config().ok_or("Failed to load config")?;
        let game_path = StardewValley::discover_path(&config).ok_or("Stardew Valley not found")?;
        let game = StardewValley::new(game_path);
        Ok(SyncManager::new(game, config))
    }
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Reconcile => match app.sync_manager() {
            Ok(sm) => {
                let path = PathBuf::from(&app.game_mod_path);
                match sm.reconcile(&path) {
                    Ok(state) => {
                        app.mod_state = state;
                        app.push_log("Reconciled mods".into());
                    }
                    Err(e) => app.push_log(format!("Reconcile failed: {e}")),
                }
            }
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::StageMod(name) => match app.sync_manager() {
            Ok(sm) => {
                let entry = app
                    .mod_state
                    .mods
                    .iter()
                    .find(|m| m.name == name)
                    .and_then(|m| m.source_entry.clone());
                match entry {
                    Some(e) => match sm.stage_one_mod(&e, &mut app.mod_state) {
                        Ok(()) => app.push_log(format!("Staged {name}")),
                        Err(e) => app.push_log(format!("Stage failed: {e}")),
                    },
                    None => app.push_log(format!("{name} not found in downloads")),
                }
            }
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::EnableMod(name) => match app.sync_manager() {
            Ok(sm) => {
                let entry = app
                    .mod_state
                    .mods
                    .iter()
                    .find(|m| m.name == name)
                    .and_then(|m| m.staging_entry.clone());
                match entry {
                    Some(e) => match sm.enable_one_mod(&e, &mut app.mod_state) {
                        Ok(()) => app.push_log(format!("Enabled {name}")),
                        Err(e) => app.push_log(format!("Enable failed: {e}")),
                    },
                    None => app.push_log(format!("{name} not found in staging")),
                }
            }
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::StageAll => match app.sync_manager() {
            Ok(sm) => match sm.stage_mods(&mut app.mod_state) {
                Ok(()) => app.push_log("All mods staged".into()),
                Err(e) => app.push_log(format!("Stage failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::EnableAll => match app.sync_manager() {
            Ok(sm) => match sm.enable_mods(&mut app.mod_state) {
                Ok(()) => app.push_log("All mods enabled".into()),
                Err(e) => app.push_log(format!("Enable failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::SyncAll => match app.sync_manager() {
            Ok(sm) => match sm.sync_all(&mut app.mod_state) {
                Ok(()) => app.push_log("Sync completed".into()),
                Err(e) => app.push_log(format!("Sync failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let action_buttons = row![
        button("Reconcile").on_press(Message::Reconcile),
        button("Stage All").on_press(Message::StageAll),
        button("Enable All").on_press(Message::EnableAll),
        button("Sync All").on_press(Message::SyncAll),
    ]
    .spacing(10);

    let mut sections = Column::new().spacing(12);

    sections = sections.push(mod_section(
        "Downloads",
        &ModStatus::Downloaded,
        "Stage",
        Message::StageMod,
        app,
    ));
    sections = sections.push(mod_section(
        "Staging",
        &ModStatus::Staged,
        "Enable",
        Message::EnableMod,
        app,
    ));
    sections = sections.push(mod_section(
        "Enabled",
        &ModStatus::Enabled,
        "",
        |_| unreachable!(),
        app,
    ));
    sections = sections.push(mod_section(
        "Modified",
        &ModStatus::Modified,
        "Re-stage",
        Message::StageMod,
        app,
    ));

    let log = scrollable(
        column(
            app.log
                .iter()
                .map(|line| text(line).into())
                .collect::<Vec<_>>(),
        )
        .spacing(4),
    )
    .height(200);

    let content = column![
        text("Moda — Mod Manager").size(24),
        text(format!("Game:           {}", app.game_path)).size(14),
        text(format!("Game mod path:  {}", app.game_mod_path)).size(14),
        text(format!("Mods path:      {}", app.mods_path)).size(14),
        text(format!("Staging path:   {}", app.staging_path)).size(14),
        action_buttons,
        sections,
        text("Log:").size(16),
        log,
    ]
    .spacing(8)
    .padding(20)
    .max_width(800);

    content.into()
}

fn mod_section<'a>(
    title: &'a str,
    status: &'a ModStatus,
    action_label: &'a str,
    action: fn(String) -> Message,
    app: &'a App,
) -> Element<'a, Message> {
    let entries: Vec<_> = app
        .mod_state
        .mods
        .iter()
        .filter(|m| &m.status == status)
        .collect();

    if entries.is_empty() {
        return text(format!("{title}: (none)")).into();
    }

    let mut col = Column::new()
        .spacing(4)
        .push(text(format!("{title} ({})", entries.len())).size(16));

    for m in entries {
        let name = text(&m.name).size(14);
        let row: Element<'_, Message> = if action_label.is_empty() {
            row![name].spacing(10).into()
        } else {
            let btn = button(action_label).on_press(action(m.name.clone()));
            row![name, btn].spacing(10).into()
        };
        col = col.push(row);
    }

    col.into()
}

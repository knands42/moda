use crate::config::load_config;
use crate::games::{Game, StardewValley};
use crate::mods::mod_registry::ModStatus;
use crate::mods::{ModState, SyncManager};
use iced::widget::{button, column, row, scrollable, text, Column};
use iced::Element;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Downloads,
    Staging,
    Enabled,
    Modified,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reconcile,
    StageMod(String),
    EnableMod(String),
    UnstageMod(String),
    DisableMod(String),
    StageAll,
    EnableAll,
    UnstageAll,
    DisableAll,
    SyncAll,
    TabSelected(Tab),
}

pub struct App {
    game_path: String,
    mods_path: String,
    staging_path: String,
    game_mod_path: String,
    mod_state: ModState,
    log: Vec<String>,
    current_tab: Tab,
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
            current_tab: Tab::Downloads,
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

pub fn update(app: &mut App, message: Message) {
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
                    .get_mod(&name)
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
                    .get_mod(&name)
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
        Message::UnstageMod(name) => match app.sync_manager() {
            Ok(sm) => {
                let entry = app
                    .mod_state
                    .get_mod(&name)
                    .and_then(|m| m.staging_entry.clone());
                match entry {
                    Some(e) => match sm.unstage_one_mod(&e, &mut app.mod_state) {
                        Ok(()) => app.push_log(format!("Unstaged {name}")),
                        Err(e) => app.push_log(format!("Unstage failed: {e}")),
                    },
                    None => app.push_log(format!("{name} not found in staging")),
                }
            }
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::DisableMod(name) => match app.sync_manager() {
            Ok(sm) => {
                let entry = app
                    .mod_state
                    .get_mod(&name)
                    .and_then(|m| m.game_entry.clone());
                match entry {
                    Some(e) => match sm.disable_one_mod(&e, &mut app.mod_state) {
                        Ok(()) => app.push_log(format!("Disabled {name}")),
                        Err(e) => app.push_log(format!("Disable failed: {e}")),
                    },
                    None => app.push_log(format!("{name} not found in game mods")),
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
        Message::UnstageAll => match app.sync_manager() {
            Ok(sm) => match sm.unstage_mods(&mut app.mod_state) {
                Ok(()) => app.push_log("All mods unstaged".into()),
                Err(e) => app.push_log(format!("Unstage failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::DisableAll => match app.sync_manager() {
            Ok(sm) => match sm.disable_mods(&mut app.mod_state) {
                Ok(()) => app.push_log("All mods disabled".into()),
                Err(e) => app.push_log(format!("Disable failed: {e}")),
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
        Message::TabSelected(tab) => app.current_tab = tab,
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let action_buttons = row![
        button("Reconcile").on_press(Message::Reconcile),
        button("Stage All").on_press(Message::StageAll),
        button("Enable All").on_press(Message::EnableAll),
        button("Unstage All").on_press(Message::UnstageAll),
        button("Disable All").on_press(Message::DisableAll),
        button("Sync All").on_press(Message::SyncAll),
    ]
    .spacing(10);

    let tabs = row![
        tab_button("Downloads", &Tab::Downloads, app),
        tab_button("Staging", &Tab::Staging, app),
        tab_button("Enabled", &Tab::Enabled, app),
        tab_button("Modified", &Tab::Modified, app),
    ]
    .spacing(4);

    let tab_content = match app.current_tab {
        Tab::Downloads => download_tab(app),
        Tab::Staging => staging_tab(app),
        Tab::Enabled => enabled_tab(app),
        Tab::Modified => modified_tab(app),
    };

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
        tabs,
        tab_content,
        text("Log:").size(16),
        log,
    ]
    .spacing(8)
    .padding(20)
    .max_width(800);

    content.into()
}

fn tab_button<'a>(label: &'a str, tab: &Tab, app: &'a App) -> Element<'a, Message> {
    let is_active = app.current_tab == *tab;
    let mut btn = button(text(label).size(16));
    if is_active {
        btn = btn.on_press(Message::TabSelected(tab.clone()));
    }
    btn.into()
}

fn download_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let mods: Vec<_> = app
        .mod_state
        .get_mods()
        .filter(|m| m.status == ModStatus::Downloaded)
        .collect();

    let mut col = Column::new()
        .spacing(4)
        .push(text(format!("Downloads ({})", mods.len())).size(16));

    if mods.is_empty() {
        col = col.push(text("(none)").size(14));
    }

    for m in &mods {
        let btn = button("Stage").on_press(Message::StageMod(m.name.clone()));
        col = col.push(row![text(&m.name).size(14), btn].spacing(10));
    }

    col.into()
}

fn staging_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let mods: Vec<_> = app
        .mod_state
        .get_mods()
        .filter(|m| m.status == ModStatus::Staged)
        .collect();

    let mut col = Column::new()
        .spacing(4)
        .push(text(format!("Staging ({})", mods.len())).size(16));

    if mods.is_empty() {
        col = col.push(text("(none)").size(14));
    }

    for m in &mods {
        let enable = button("Enable").on_press(Message::EnableMod(m.name.clone()));
        let unstage = button("Unstage").on_press(Message::UnstageMod(m.name.clone()));
        col = col.push(row![text(&m.name).size(14), enable, unstage].spacing(10));
    }

    col.into()
}

fn enabled_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let mods: Vec<_> = app
        .mod_state
        .get_mods()
        .filter(|m| m.status == ModStatus::Enabled)
        .collect();

    let mut col = Column::new()
        .spacing(4)
        .push(text(format!("Enabled ({})", mods.len())).size(16));

    if mods.is_empty() {
        col = col.push(text("(none)").size(14));
    }

    for m in &mods {
        let btn = button("Disable").on_press(Message::DisableMod(m.name.clone()));
        col = col.push(row![text(&m.name).size(14), btn].spacing(10));
    }

    col.into()
}

fn modified_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let mods: Vec<_> = app
        .mod_state
        .get_mods()
        .filter(|m| m.status == ModStatus::Modified)
        .collect();

    let mut col = Column::new()
        .spacing(4)
        .push(text(format!("Modified ({})", mods.len())).size(16));

    if mods.is_empty() {
        col = col.push(text("(none)").size(14));
    }

    for m in &mods {
        let restage = button("Re-stage").on_press(Message::StageMod(m.name.clone()));
        let disable = button("Disable").on_press(Message::DisableMod(m.name.clone()));
        col = col.push(row![text(&m.name).size(14), restage, disable].spacing(10));
    }

    col.into()
}

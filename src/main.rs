use iced::widget::{button, column, row, scrollable, text};
use iced::Element;
use moda::config::load_config;
use moda::games::{Game, StardewValley};
use moda::mods::SyncManager;
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::run("Moda", update, view)
}

#[derive(Debug, Clone)]
enum Message {
    StageMods,
    EnableMods,
    SyncAll,
}

struct App {
    game_path: String,
    mods_path: String,
    staging_path: String,
    game_mod_path: String,
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
            log: vec!["App started".into()],
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
        Message::StageMods => match app.sync_manager() {
            Ok(sm) => match sm.stage_mods() {
                Ok(()) => app.push_log("Mods staged successfully".into()),
                Err(e) => app.push_log(format!("Stage failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::EnableMods => match app.sync_manager() {
            Ok(sm) => match sm.enable_mods() {
                Ok(()) => app.push_log("Mods enabled successfully".into()),
                Err(e) => app.push_log(format!("Enable failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
        Message::SyncAll => match app.sync_manager() {
            Ok(sm) => match sm.sync_all() {
                Ok(()) => app.push_log("Sync completed successfully".into()),
                Err(e) => app.push_log(format!("Sync failed: {e}")),
            },
            Err(e) => app.push_log(format!("Init failed: {e}")),
        },
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let buttons = row![
        button("Stage Mods").on_press(Message::StageMods),
        button("Enable Mods").on_press(Message::EnableMods),
        button("Sync All").on_press(Message::SyncAll),
    ]
    .spacing(10);

    let log = scrollable(
        column(
            app.log
                .iter()
                .map(|line| text(line).into())
                .collect::<Vec<_>>(),
        )
        .spacing(4),
    )
    .height(300);

    let content = column![
        text("Moda — Mod Manager").size(24),
        text(format!("Game path:      {}", app.game_path)).size(14),
        text(format!("Game mod path:  {}", app.game_mod_path)).size(14),
        text(format!("Mods path:      {}", app.mods_path)).size(14),
        text(format!("Staging path:   {}", app.staging_path)).size(14),
        buttons,
        text("Log:").size(16),
        log,
    ]
    .spacing(8)
    .padding(20)
    .max_width(800);

    content.into()
}

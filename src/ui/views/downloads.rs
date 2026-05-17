use crate::mods::catalog::ModStatus;
use crate::ui::app::App;
use crate::ui::components::mod_row;
use crate::ui::message::Message;
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let mut mods: Vec<_> = app
        .mod_state
        .get_mods()
        .filter(|m| m.status == ModStatus::Downloaded)
        .collect();
    mods.sort_by(|a, b| a.name.cmp(&b.name));

    let count = mods.len();

    let header = column![
        text("Downloads").size(22),
        text(format!(
            "{count} mod{} ready to stage",
            if count == 1 { "" } else { "s" }
        ))
        .size(13),
    ]
    .spacing(2);

    let body: Element<'_, Message> = if mods.is_empty() {
        container(
            text("No downloaded mods. Drop .zip files or directories into your mods folder, then click Reconcile.")
                .size(13),
        )
        .padding(24)
        .width(Length::Fill)
        .into()
    } else {
        let rows = column(
            mods.into_iter()
                .map(mod_row::download_row)
                .collect::<Vec<_>>(),
        )
        .spacing(6);
        scrollable(rows).height(Length::Fill).into()
    };

    column![header, body]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

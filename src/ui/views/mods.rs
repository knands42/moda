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
        .filter(|m| {
            matches!(
                m.status,
                ModStatus::Staged | ModStatus::Enabled | ModStatus::Modified
            )
        })
        .collect();
    mods.sort_by(|a, b| a.name.cmp(&b.name));

    let enabled_count = mods
        .iter()
        .filter(|m| m.status == ModStatus::Enabled)
        .count();
    let total = mods.len();

    let header = column![
        text("Mods").size(22),
        text(format!("{enabled_count} enabled · {total} staged")).size(13),
    ]
    .spacing(2);

    let body: Element<'_, Message> = if mods.is_empty() {
        container(
            text("No staged mods yet. Stage them from the Downloads tab to manage here.").size(13),
        )
        .padding(24)
        .width(Length::Fill)
        .into()
    } else {
        let rows = column(
            mods.into_iter()
                .map(mod_row::staged_row)
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

use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;
use crate::ui::styles;

const MAX_VISIBLE_LINES: usize = 50;

pub fn view(app: &App) -> Element<'_, Message> {
    let start = app.log.len().saturating_sub(MAX_VISIBLE_LINES);
    let entries = column(
        app.log[start..]
            .iter()
            .map(|line| text(line.as_str()).size(12).into())
            .collect::<Vec<_>>(),
    )
    .spacing(3);

    let inner = column![
        text("Activity").size(13),
        scrollable(entries).height(120).width(Length::Fill),
    ]
    .spacing(8)
    .padding(14);

    container(inner)
        .width(Length::Fill)
        .style(styles::card)
        .into()
}

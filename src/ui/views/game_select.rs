use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};

use crate::games;
use crate::ui::app::App;
use crate::ui::message::Message;
use crate::ui::styles;

pub fn view(_app: &App) -> Element<'_, Message> {
    let games = games::supported_games();

    let title = column![
        text("Welcome to Moda").size(28),
        text("Select a game to get started").size(14),
    ]
    .spacing(4)
    .width(Length::Fill)
    .align_x(iced::Alignment::Center);

    let cards: Vec<Element<'_, Message>> = games
        .into_iter()
        .map(|(id, name, description)| {
            let card = column![text(name).size(20), text(description).size(13),]
                .spacing(6)
                .padding(20);

            button(card)
                .width(Length::Fill)
                .padding(0)
                .style(styles::game_card)
                .on_press(Message::SelectGame(id.to_string()))
                .into()
        })
        .collect();

    let grid = column(cards).spacing(12).padding(20).max_width(520);

    let body = scrollable(
        column![title, grid]
            .spacing(32)
            .align_x(iced::Alignment::Center)
            .width(Length::Fill),
    )
    .height(Length::Fill);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::main_bg)
        .into()
}

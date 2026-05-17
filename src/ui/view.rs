use iced::widget::{column, container, row, Space};
use iced::{Element, Length};

use super::app::App;
use super::components::{log_panel, sidebar};
use super::message::{Message, Tab};
use super::styles;
use super::views;

pub fn view(app: &App) -> Element<'_, Message> {
    if app.current_tab == Tab::GameSelect {
        return views::game_select::view(app);
    }

    let body: Element<'_, Message> = match app.current_tab {
        Tab::Downloads => views::downloads::view(app),
        Tab::Mods => views::mods::view(app),
        _ => unreachable!(),
    };

    let main_column = column![body, Space::with_height(8), log_panel::view(app)]
        .spacing(8)
        .padding(24);

    let main_area = container(main_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::main_bg);

    row![sidebar::view(app), main_area]
        .height(Length::Fill)
        .into()
}

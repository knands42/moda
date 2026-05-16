use iced::widget::{button, column, container, text, Space};
use iced::{Element, Length, Theme};

use crate::ui::app::App;
use crate::ui::message::{Message, Tab};
use crate::ui::styles;

const SIDEBAR_WIDTH: u16 = 220;

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text("Moda").size(28);
    let subtitle = text(app.game_name.as_str())
        .size(13)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let nav = column![
        tab_button("Mods", Tab::Mods, app.current_tab),
        tab_button("Downloads", Tab::Downloads, app.current_tab),
    ]
    .spacing(4);

    let actions = column![
        button(text("Reconcile").size(14))
            .width(Length::Fill)
            .padding([8, 12])
            .style(iced::widget::button::secondary)
            .on_press(Message::Reconcile),
        button(text("Sync All").size(14))
            .width(Length::Fill)
            .padding([8, 12])
            .style(iced::widget::button::primary)
            .on_press(Message::SyncAll),
    ]
    .spacing(8);

    let content = column![
        title,
        subtitle,
        Space::with_height(24),
        nav,
        Space::with_height(Length::Fill),
        actions,
    ]
    .spacing(6)
    .padding(20)
    .width(SIDEBAR_WIDTH);

    container(content)
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .style(styles::sidebar_bg)
        .into()
}

fn tab_button<'a>(label: &'a str, tab: Tab, current: Tab) -> Element<'a, Message> {
    let is_active = current == tab;
    let style: fn(&Theme, button::Status) -> button::Style = if is_active {
        styles::tab_active
    } else {
        styles::tab_inactive
    };

    button(text(label).size(15))
        .width(Length::Fill)
        .padding([10, 14])
        .style(style)
        .on_press(Message::TabSelected(tab))
        .into()
}

use iced::widget::{button, column, container, row, text, Space};
use iced::{font::Family, Element, Font, Length, Theme};

use crate::ui::app::App;
use crate::ui::message::{Message, Tab};
use crate::ui::styles;

const SIDEBAR_WIDTH: u16 = 220;

pub fn view(app: &App) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = vec![text("Moda").size(28).into()];

    if app.current_tab != Tab::GameSelect {
        let subtitle = text(app.game_name.as_str())
            .size(13)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.extended_palette().background.base.text),
            });
        items.push(subtitle.into());
        items.push(Space::with_height(8).into());

        let emoji = text("🔙").size(14).font(Font {
            family: Family::Name("Noto Emoji"),
            ..Default::default()
        });
        let back = button(row![emoji, text(" Change Game").size(14)].spacing(0))
            .width(Length::Fill)
            .padding([6, 10])
            .style(iced::widget::button::text)
            .on_press(Message::TabSelected(Tab::GameSelect));
        items.push(back.into());
        items.push(Space::with_height(16).into());

        let nav = column![
            tab_button("Mods", Tab::Mods, app.current_tab),
            tab_button("Downloads", Tab::Downloads, app.current_tab),
        ]
        .spacing(4);
        items.push(nav.into());
        items.push(Space::with_height(Length::Fill).into());

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
        items.push(actions.into());
    } else {
        items.push(Space::with_height(Length::Fill).into());
    }

    let content = column(items).spacing(6).padding(20).width(SIDEBAR_WIDTH);

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

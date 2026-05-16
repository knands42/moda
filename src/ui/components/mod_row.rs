use iced::widget::{button, container, row, text, toggler, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::mods::mod_registry::{ModStatus, ReconciledMod};
use crate::ui::message::Message;
use crate::ui::styles;

pub fn download_row(m: &ReconciledMod) -> Element<'_, Message> {
    let stage_btn = button(text("Stage").size(13))
        .padding([6, 14])
        .style(iced::widget::button::primary)
        .on_press(Message::StageMod(m.name.clone()));

    let inner = row![
        text(m.name.as_str()).size(14).width(Length::Fill),
        status_badge(m.status),
        stage_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([10, 14]);

    container(inner)
        .width(Length::Fill)
        .style(styles::list_row)
        .into()
}

pub fn staged_row(m: &ReconciledMod) -> Element<'_, Message> {
    let is_enabled = m.status == ModStatus::Enabled;
    let is_modified = m.status == ModStatus::Modified;

    let toggle: Element<'_, Message> = if is_modified {
        button(text("Re-stage").size(13))
            .padding([6, 14])
            .style(iced::widget::button::primary)
            .on_press(Message::StageMod(m.name.clone()))
            .into()
    } else {
        let name = m.name.clone();
        toggler(is_enabled)
            .size(20)
            .on_toggle(move |enabled| {
                if enabled {
                    Message::EnableMod(name.clone())
                } else {
                    Message::DisableMod(name.clone())
                }
            })
            .into()
    };

    let unstage_btn = button(text("Unstage").size(13))
        .padding([6, 14])
        .style(iced::widget::button::danger)
        .on_press(Message::UnstageMod(m.name.clone()));

    let inner = row![
        text(m.name.as_str()).size(14).width(Length::Fill),
        status_badge(m.status),
        Space::with_width(4),
        toggle,
        unstage_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([10, 14]);

    container(inner)
        .width(Length::Fill)
        .style(styles::list_row)
        .into()
}

fn status_badge<'a>(status: ModStatus) -> Element<'a, Message> {
    let (label, style): (&'static str, fn(&Theme) -> container::Style) = match status {
        ModStatus::Enabled => ("Enabled", styles::badge_enabled),
        ModStatus::Staged => ("Staged", styles::badge_staged),
        ModStatus::Modified => ("Modified", styles::badge_modified),
        ModStatus::Downloaded => ("Downloaded", styles::badge_downloaded),
    };

    container(text(label).size(11))
        .padding([3, 10])
        .style(style)
        .into()
}

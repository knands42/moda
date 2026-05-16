use moda::ui::{update, view};

fn main() -> iced::Result {
    iced::application("Moda", update, view)
        .theme(|_| iced::Theme::CatppuccinMocha)
        .run()
}

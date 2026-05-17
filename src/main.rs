use moda::ui::{update, view};

fn main() -> iced::Result {
    env_logger::init();
    log_panics::init();

    log::info!("Starting Moda");

    iced::application("Moda", update, view)
        .theme(|_| iced::Theme::CatppuccinMocha)
        .run()
}

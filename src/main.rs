use eframe::egui;

fn main() {
    env_logger::init();
    log_panics::init();

    log::info!("Starting Moda");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Moda",
        options,
        Box::new(|_cc| Ok(Box::new(moda::ui::app::ModaApp::new()))),
    )
    .expect("Failed to start Moda");
}

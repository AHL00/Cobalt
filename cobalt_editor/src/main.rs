// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 

fn main() -> eframe::Result<()> {
    use log::LevelFilter;
    use simple_logger::SimpleLogger;

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cobalt Editor",
        native_options,
        Box::new(|cc| Box::new(cobalt_editor::App::new(cc))),
    )
}

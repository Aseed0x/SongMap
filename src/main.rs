#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod types;

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("icone invalide");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SongMap")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "SongMap",
        options,
        Box::new(|cc| Ok(Box::new(app::SongMapApp::new(cc)))),
    )
}

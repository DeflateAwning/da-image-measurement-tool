// da-image-measurement-tool
//
// Load an image, calibrate it by marking two points of a KNOWN real-world
// length, then click arbitrary point pairs anywhere on the image to read off
// their real-world distance. Useful for reverse-engineering part dimensions
// from a photo (e.g. a photo of a PCB next to a ruler, or a part drawing
// with one known dimension).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod geometry;
mod model;
mod export;

use app::MeasurementApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("DA Image Measurement Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "da-image-measurement-tool",
        native_options,
        Box::new(|cc| Box::new(MeasurementApp::new(cc))),
    )
}

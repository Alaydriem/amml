//! The amml-studio-ui gallery: one page per Paper artboard group, drawn with the real components.

mod gallery_app;
mod gallery_id;

use gallery_app::GalleryApp;

struct Gallery;

impl Gallery {
    fn run() -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1480.0, 1000.0])
                .with_title("studio-ui gallery"),
            ..Default::default()
        };
        eframe::run_native("studio-ui-gallery", options, Box::new(|cc| Ok(Box::new(GalleryApp::new(&cc.egui_ctx)))))
    }
}

fn main() -> eframe::Result {
    Gallery::run()
}

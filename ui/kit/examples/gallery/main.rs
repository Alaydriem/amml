//! The amml-ui gallery: one page per Paper widget sheet, drawn with the real widgets.

mod gallery_app;
mod gallery_id;

use gallery_app::GalleryApp;

struct Gallery;

impl Gallery {
    fn run() -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1480.0, 1000.0]).with_title("amml-ui gallery"),
            ..Default::default()
        };
        eframe::run_native("amml-ui-gallery", options, Box::new(|cc| Ok(Box::new(GalleryApp::new(&cc.egui_ctx)))))
    }
}

fn main() -> eframe::Result {
    Gallery::run()
}

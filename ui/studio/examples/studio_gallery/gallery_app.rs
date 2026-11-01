use amml_studio_ui::gallery::{GalleryPage, GalleryState};
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::{CentralPanel, Context, Panel, ScrollArea};

use crate::gallery_id::GalleryId;

// a page list on the left and the chosen Paper artboard on the right
pub struct GalleryApp {
    page: Option<GalleryPage>,
    state: GalleryState,
}

impl GalleryApp {
    const LIST_WIDTH: f32 = 200.0;

    pub fn new(ctx: &Context) -> Self {
        Theme::install(ctx, FontSource::Embedded);
        Self { page: GalleryPage::ALL.first().copied(), state: GalleryState::default() }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Panel::left(GalleryId::Pages).exact_size(Self::LIST_WIDTH).show(ui, |ui| {
            for page in GalleryPage::ALL {
                if ui.selectable_label(self.page == Some(page), page.name()).clicked() {
                    self.page = Some(page);
                }
            }
        });
        CentralPanel::default().show(ui, |ui| {
            if let Some(page) = self.page {
                ScrollArea::both().show(ui, |ui| page.show(ui, &mut self.state));
            }
        });
    }
}

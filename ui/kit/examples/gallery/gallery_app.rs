use amml_ui::font::FontSource;
use amml_ui::gallery::{GallerySheet, GalleryState};
use amml_ui::theme::Theme;
use egui::{CentralPanel, Context, Panel, ScrollArea};

use crate::gallery_id::GalleryId;

// a page list on the left and the chosen Paper sheet on the right
pub struct GalleryApp {
    sheet: GallerySheet,
    state: GalleryState,
}

impl GalleryApp {
    const LIST_WIDTH: f32 = 200.0;

    pub fn new(ctx: &Context) -> Self {
        Theme::install(ctx, FontSource::Embedded);
        Self { sheet: GallerySheet::TextButton, state: GalleryState::default() }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Panel::left(GalleryId::Pages).exact_size(Self::LIST_WIDTH).show(ui, |ui| {
            for sheet in GallerySheet::ALL {
                if ui.selectable_label(self.sheet == sheet, sheet.name()).clicked() {
                    self.sheet = sheet;
                }
            }
        });
        CentralPanel::default().show(ui, |ui| {
            ScrollArea::both().show(ui, |ui| self.sheet.show(ui, &mut self.state));
        });
    }
}

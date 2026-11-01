use egui::Ui;

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::widget::{ToastLine, ToastTone};

/// Paper: Widgets, `Widget / Toast · one line` (`GCO-0`).
pub struct ToastLineSheet;

impl ToastLineSheet {
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Toast · one line", |ui| {
            SheetGrid::panel(ui, |ui| {
                let samples = [
                    ("Info", ToastTone::Info, "Added Arrival to the render queue"),
                    ("Error", ToastTone::Danger, "Select two or more keys at different times first"),
                ];
                for (label, tone, words) in samples {
                    let line = ToastLine::new(tone, Sample::text(words));
                    let size = line.desired_size(ui.ctx());
                    SheetGrid::labelled(ui, label, SheetGrid::LABEL_WIDTH, SheetGrid::COLUMN_GAP, size, |ui, rect| {
                        line.paint(ui.painter(), rect.min);
                    });
                }
            });
        });
    }
}

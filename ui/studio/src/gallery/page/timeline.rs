use egui::Ui;

use super::{TimelineSamples, ViewerSamples};
use crate::component::timeline::TimelineView;
use crate::gallery::GalleryState;

/// Paper: Editor components, `Editor / Timeline · assembled` (`54P-0`). The whole timeline on the Arrival scene.
pub struct TimelinePage;

impl TimelinePage {
    /// Paper: the timeline frame is 394 high.
    const HEIGHT: f32 = 394.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        super::BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "TIMELINE · ASSEMBLED", |ui| {
                let model = TimelineSamples::model();
                TimelineSamples::place(ui, Self::HEIGHT, |ui, _| TimelineView::new(&model).show(ui));
            });
        });
    }
}

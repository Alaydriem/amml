use egui::{Ui, vec2};

use crate::gallery::{GalleryState, SheetGrid};
use crate::state::InteractionState;
use crate::widget::Slider;

/// Paper: Widgets, `Widget / Slider` (`AHS-0`). Rows are states; the sheet uses its own label
/// column (130), label gap (20) and row gap (18).
pub struct SliderSheet;

impl SliderSheet {
    const LABEL_WIDTH: f32 = 130.0;
    const LABEL_GAP: f32 = 20.0;
    const ROW_GAP: f32 = 18.0;
    const WIDTH: f32 = 240.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let rows = [
            ("Rest", 0.4, InteractionState::Rest),
            ("Hover", 0.4, InteractionState::Hover),
            ("Dragging", 0.7, InteractionState::Pressed),
            ("Focus", 0.4, InteractionState::Focused),
            ("Disabled", 0.4, InteractionState::Disabled),
        ];
        SheetGrid::page(ui, "Slider", |ui| {
            SheetGrid::panel_with_gap(ui, Self::ROW_GAP, |ui| {
                for (label, value, state) in rows {
                    let size = vec2(Self::WIDTH, Slider::HEIGHT);
                    SheetGrid::labelled(ui, label, Self::LABEL_WIDTH, Self::LABEL_GAP, size, |ui, rect| {
                        let mut value = value;
                        Slider::new(&mut value, 0.0..=1.0).paint(ui.painter(), rect, state);
                    });
                }
                let size = vec2(Self::WIDTH, Slider::HEIGHT_WITH_TICKS);
                SheetGrid::labelled(ui, "With steps", Self::LABEL_WIDTH, Self::LABEL_GAP, size, |ui, rect| {
                    let mut value = 3.0;
                    Slider::new(&mut value, 0.0..=4.0).step(1.0).ticks(true).paint(
                        ui.painter(),
                        rect,
                        InteractionState::Rest,
                    );
                });
            });
        });
    }
}

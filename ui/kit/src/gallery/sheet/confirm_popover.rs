use egui::{Rect, Sense, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::widget::ConfirmPopover;

/// Paper: Widgets, `Widget / Popover · Confirm`. The discard question at rest, with its danger button hovered, and
/// with it focused.
pub struct ConfirmPopoverSheet;

impl ConfirmPopoverSheet {
    /// Sheet chrome: the column each state sits in.
    const COLUMN: f32 = 360.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Popover · Confirm", |ui| {
            SheetGrid::panel(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                    let states = [
                        ("REST", InteractionState::Rest),
                        ("CONFIRM HOVER", InteractionState::Hover),
                        ("FOCUS", InteractionState::Focused),
                    ];
                    for (head, state) in states {
                        SheetGrid::column(ui, head, Self::COLUMN, |ui| {
                            let popover =
                                ConfirmPopover::new(Sample::text("Discard this recording?"), Sample::text("Discard"))
                                    .body(Sample::text(
                                        "12:41 of capture and 41.2 MB will be deleted. You cannot undo this.",
                                    ))
                                    .cancel(Sample::text("Keep recording"));
                            let size = popover.size(ui.ctx());
                            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                            popover.paint(ui.painter(), Rect::from_min_size(rect.min, size), state);
                        });
                    }
                });
            });
        });
    }
}

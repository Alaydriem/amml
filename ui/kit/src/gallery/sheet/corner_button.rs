use amml_i18n::text::UserText;
use egui::{Sense, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{CornerButton, CornerButtonLead};

/// Paper: Entry Point, `Components · capture rail & corner buttons`, row "Corner buttons" (`AKS-0`). A mod's entry
/// and the Mods button, side by side, in each look.
pub struct CornerButtonSheet;

impl CornerButtonSheet {
    /// Sheet chrome: a column's width.
    const COLUMN: f32 = 220.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Corner buttons", |ui| {
            SheetGrid::panel(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP + SheetGrid::COLUMN_GAP, 0.0);
                    let looks = [
                        ("DEFAULT", InteractionState::Rest),
                        ("HOVER", InteractionState::Hover),
                        ("PRESSED", InteractionState::Pressed),
                        ("FOCUS", InteractionState::Focused),
                    ];
                    for (head, state) in looks {
                        SheetGrid::column(ui, head, Self::COLUMN, |ui| Self::pair(ui, state));
                    }
                });
            });
        });
    }

    // the Dejavu entry, then the Mods button, `--spacing-s` apart
    fn pair(ui: &mut Ui, state: InteractionState) {
        let entry = CornerButton::new(CornerButtonLead::Mark(UserText::from_model("DV".to_owned())))
            .label(Sample::text("Dejavu"));
        let mods = CornerButton::new(CornerButtonLead::Icon(Icon::Mods));
        let gap = <spacing::S as SpacingToken>::VALUE;
        let entry_width = entry.width(ui.ctx());
        let size = vec2(entry_width + gap + CornerButton::HEIGHT, CornerButton::HEIGHT);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let (left, right) = rect.split_left_right_at_x(rect.left() + entry_width);
        entry.paint(ui.painter(), left, state);
        let side = vec2(CornerButton::HEIGHT, CornerButton::HEIGHT);
        let square = egui::Rect::from_min_size(right.right_top() - vec2(CornerButton::HEIGHT, 0.0), side);
        // Paper's focus cell rings the entry only
        let mods_state = if state == InteractionState::Focused { InteractionState::Rest } else { state };
        mods.paint(ui.painter(), square, mods_state);
    }
}

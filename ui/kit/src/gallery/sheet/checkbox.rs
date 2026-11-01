use egui::{Sense, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::{InteractionState, Selection};
use crate::tokens::{SpacingToken, spacing};
use crate::widget::Checkbox;

/// Paper: Widgets, `Widget / Checkbox` (`54Q-0`).
pub struct CheckboxSheet;

impl CheckboxSheet {
    const COLUMN: f32 = 120.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let side = <spacing::L as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Checkbox", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, value) in
                    [("Unchecked", Selection::Off), ("Checked", Selection::On), ("Mixed", Selection::Mixed)]
                {
                    SheetGrid::row(ui, label, Self::COLUMN, side, Self::STATES.len(), |ui, index, cell| {
                        let mut value = value;
                        let rect = SheetGrid::place(cell, vec2(side, side));
                        Checkbox::new(&mut value).paint(ui.painter(), rect, Self::STATES[index]);
                    });
                }
            });
            SheetGrid::section(ui, "IN A LIST", |ui| {
                SheetGrid::list_panel(ui, |ui| {
                    let rows = [
                        ("Record tracks", Selection::Mixed, false, InteractionState::Hover),
                        ("Camera", Selection::On, true, InteractionState::Rest),
                        ("FOV", Selection::Off, true, InteractionState::Rest),
                        ("Weather", Selection::On, true, InteractionState::Rest),
                        ("Timelapse", Selection::Off, true, InteractionState::Disabled),
                    ];
                    for (label, value, indent, state) in rows {
                        let height = <spacing::Field as SpacingToken>::VALUE;
                        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
                        let mut value = value;
                        Checkbox::new(&mut value).label(Sample::text(label)).indent(indent).paint_row(
                            ui.painter(),
                            rect,
                            state,
                        );
                    }
                });
            });
        });
    }
}

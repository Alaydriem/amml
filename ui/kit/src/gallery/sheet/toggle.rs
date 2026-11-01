use egui::{Sense, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{SettingControl, SettingRow, Toggle};

/// Paper: Widgets, `Widget / Toggle` (`AG6-0`).
pub struct ToggleSheet;

impl ToggleSheet {
    const COLUMN: f32 = 120.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Toggle", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, on) in [("Off", false), ("On", true)] {
                    SheetGrid::row(ui, label, Self::COLUMN, Toggle::TRACK.y, Self::STATES.len(), |ui, index, cell| {
                        let mut value = on;
                        let rect = SheetGrid::place(cell, Toggle::TRACK);
                        Toggle::new(&mut value).paint(ui.painter(), rect, Self::STATES[index]);
                    });
                }
            });
            SheetGrid::section(ui, "IN A SETTING ROW", |ui| {
                SheetGrid::list_panel(ui, |ui| {
                    let rows = [
                        ("Hide chat", true, InteractionState::Hover),
                        ("Hide commands", false, InteractionState::Rest),
                    ];
                    for (label, on, state) in rows {
                        let height = <spacing::Field as SpacingToken>::VALUE;
                        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
                        let mut value = on;
                        SettingRow::new(Sample::text(label), SettingControl::Toggle(&mut value)).paint(
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

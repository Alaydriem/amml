use egui::{Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::{DisabledReason, InteractionState};
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{Segment, Segmented};

/// Paper: Widgets, `Widget / Segmented` (`5MX-0`).
pub struct SegmentedSheet;

impl SegmentedSheet {
    const COLUMN: f32 = 180.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::Field as SpacingToken>::VALUE;
        let words =
            |list: &[&str]| -> Vec<Segment> { list.iter().map(|word| Segment::Text(Sample::text(word))).collect() };
        SheetGrid::page(ui, "Segmented", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                // the state goes on the selected "60" in the first row and on "24" in the second
                for (label, target) in [("Selected", 2), ("Unselected", 0)] {
                    SheetGrid::row(ui, label, Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                        let mut selected = 2;
                        let control = Segmented::new(egui::Id::NULL, &mut selected, words(&["24", "30", "60"]));
                        control.paint(
                            ui.painter(),
                            SheetGrid::place(cell, vec2(Self::COLUMN, height)),
                            target,
                            Self::STATES[index],
                        );
                    });
                }
                SheetGrid::row(ui, "Icons", Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                    let mut selected = 1;
                    let control = Segmented::new(
                        egui::Id::NULL,
                        &mut selected,
                        vec![Segment::Icon(Icon::List), Segment::Icon(Icon::Grid)],
                    );
                    let width = control.icon_width();
                    control.paint(ui.painter(), SheetGrid::place(cell, vec2(width, height)), 0, Self::STATES[index]);
                });
                // Paper row `One disabled`: "24" cannot be chosen; the hover is on it, the press and the focus on "30"
                SheetGrid::row(ui, "One disabled", Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                    let mut selected = 2;
                    let control = Segmented::new(egui::Id::NULL, &mut selected, words(&["24", "30", "60"]))
                        .disable_segment(0, DisabledReason::new(Sample::text("Not on this computer")));
                    let target = if Self::STATES[index] == InteractionState::Hover { 0 } else { 1 };
                    control.paint(
                        ui.painter(),
                        SheetGrid::place(cell, vec2(Self::COLUMN, height)),
                        target,
                        Self::STATES[index],
                    );
                });
            });
            SheetGrid::section(ui, "IN USE", |ui| {
                SheetGrid::panel(ui, |ui| {
                    let rows: [(&str, &[&str], usize, f32); 3] = [
                        ("Camera path", &["Linear", "Smooth"], 1, 240.0),
                        ("Weather", &["Recorded", "Clear", "Rain", "Thunder"], 0, 400.0),
                        ("Time of day", &["Recorded", "Sunrise", "Noon", "Sunset", "Night"], 2, 480.0),
                    ];
                    for (label, list, selected, width) in rows {
                        SheetGrid::row(ui, label, width, height, 1, |ui, _, cell| {
                            let mut choice = selected;
                            let control = Segmented::new(egui::Id::NULL, &mut choice, words(list));
                            control.paint(ui.painter(), cell, selected, InteractionState::Rest);
                        });
                    }
                });
            });
        });
    }
}

use egui::{Id, Ui, pos2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{Dropdown, DropdownOption};

/// Paper: Widgets, `Widget / Dropdown` (`3I6-0`).
pub struct DropdownSheet;

impl DropdownSheet {
    const COLUMN: f32 = 180.0;
    /// Sheet chrome: the OPEN example's width and the space its list takes under it.
    const OPEN_WIDTH: f32 = 220.0;
    const OPEN_HEIGHT: f32 = 188.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::Field as SpacingToken>::VALUE;
        let weather = || {
            let mut options: Vec<DropdownOption> = ["Recorded", "Clear", "Rain", "Thunder"]
                .iter()
                .map(|word| DropdownOption::new(Sample::text(word)))
                .collect();
            options.push(DropdownOption::new(Sample::text("Snow")).enabled(false));
            options
        };
        SheetGrid::page(ui, "Dropdown", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, selected) in [("Value", Some(2)), ("Placeholder", None)] {
                    SheetGrid::row(ui, label, Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                        let mut selected = selected;
                        Dropdown::new(Id::NULL, &mut selected, weather())
                            .placeholder(Sample::text("Choose a scene"))
                            .paint(ui.painter(), cell, Self::STATES[index], false);
                    });
                }
            });
            SheetGrid::section(ui, "OPEN", |ui| {
                SheetGrid::panel(ui, |ui| {
                    // the label lines up with the trigger; the open list hangs below the row
                    SheetGrid::row(ui, "Weather", Self::OPEN_WIDTH, height, 1, |ui, _, trigger| {
                        let mut selected = Some(2);
                        let dropdown = Dropdown::new(Id::NULL, &mut selected, weather());
                        dropdown.paint(ui.painter(), trigger, InteractionState::Rest, true);
                        let top = pos2(trigger.left(), trigger.bottom() + <spacing::Xs as SpacingToken>::VALUE);
                        dropdown.paint_list(ui.painter(), top, Self::OPEN_WIDTH, Some(1));
                    });
                    ui.add_space(Self::OPEN_HEIGHT - height - ui.spacing().item_spacing.y);
                });
            });
        });
    }
}

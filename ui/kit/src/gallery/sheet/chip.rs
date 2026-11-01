use egui::{Ui, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Chip, ChipKind, ChipTone, KeyCapSize, KeyChord};

/// Paper: Widgets, `Widget / Chip + Chip style` (`2PH-0`).
pub struct ChipSheet;

impl ChipSheet {
    const COLUMN: f32 = 120.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::ControlSmall as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Chip + Chip style", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, selected) in [("Filter", false), ("Selected", true)] {
                    SheetGrid::row(ui, label, Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                        Self::paint(
                            ui,
                            cell,
                            &Chip::new(ChipKind::Filter { selected }, Sample::text("Camera")),
                            Self::STATES[index],
                        );
                    });
                }
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["NEUTRAL", "ACCENT", "WARNING", "DANGER", "SUCCESS", "OUTLINE"]);
                let style = [
                    (ChipTone::Neutral, "Camera"),
                    (ChipTone::Accent, "Active"),
                    (ChipTone::Warning, "Shared"),
                    (ChipTone::Danger, "Conflict"),
                    (ChipTone::Success, "Granted"),
                    (ChipTone::Outline, "Saving…"),
                ];
                SheetGrid::row(ui, "Style", Self::COLUMN, height, style.len(), |ui, index, cell| {
                    let (tone, words) = style[index];
                    Self::paint(
                        ui,
                        cell,
                        &Chip::new(ChipKind::Display(tone), Sample::text(words)),
                        InteractionState::Rest,
                    );
                });
                let with_icon = [
                    Some((ChipTone::Neutral, Icon::Lock, "Locked")),
                    None,
                    None,
                    Some((ChipTone::Danger, Icon::Blocked, "Blocked")),
                    None,
                    Some((ChipTone::Outline, Icon::Info, "Reload")),
                ];
                SheetGrid::row(ui, "With icon", Self::COLUMN, height, with_icon.len(), |ui, index, cell| {
                    if let Some((tone, icon, words)) = with_icon[index] {
                        let chip = Chip::new(ChipKind::Display(tone), Sample::text(words)).icon(icon);
                        Self::paint(ui, cell, &chip, InteractionState::Rest);
                    }
                });
            });
            SheetGrid::panel(ui, |ui| {
                let cap = KeyCapSize::Regular.height();
                SheetGrid::row(ui, "Key chord", Self::COLUMN * 4.0, cap, 1, |ui, _, cell| {
                    let chord = KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("Shift"), Sample::text("N")]);
                    chord.paint(ui.painter(), cell.left_center());
                });
                SheetGrid::row(ui, "Conflict", Self::COLUMN * 4.0, cap, 1, |ui, _, cell| {
                    let chord = KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("D")]).conflict(true);
                    let rect = chord.paint(ui.painter(), cell.left_center());
                    let danger = <color::Danger as ColorToken>::VALUE;
                    let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
                    let galley = Draw::galley(ui.painter(), "Used by Duplicate scene", style, danger, TextFit::Line);
                    let x = rect.right() + <spacing::M as SpacingToken>::VALUE;
                    ui.painter().galley(pos2(x, cell.center().y - galley.size().y / 2.0), galley, danger);
                });
            });
        });
    }

    fn paint(ui: &Ui, cell: egui::Rect, chip: &Chip, state: InteractionState) {
        let size = vec2(chip.desired_width(ui.ctx()), <spacing::ControlSmall as SpacingToken>::VALUE);
        chip.paint(ui.painter(), SheetGrid::place(cell, size), state);
    }
}

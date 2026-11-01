use egui::{Ui, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{ColorToken, color};
use crate::widget::{IconButton, IconButtonSize};

/// Paper: Widgets, `Widget / Icon button` (`AEK-0`).
pub struct IconButtonSheet;

impl IconButtonSheet {
    const COLUMN: f32 = 120.0;
    const LEGEND_GAP: f32 = 12.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = IconButtonSize::Regular.side();
        SheetGrid::page(ui, "Icon button", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, on) in [("Plain", false), ("Toggled on", true)] {
                    SheetGrid::row(ui, label, Self::COLUMN, height, Self::STATES.len(), |ui, index, cell| {
                        let rect = SheetGrid::place(cell, vec2(height, height));
                        IconButton::new(Icon::Loop).on(on).paint(ui.painter(), rect, Self::STATES[index]);
                    });
                }
                let sizes =
                    [(Icon::Undo, IconButtonSize::Regular, "Regular"), (Icon::Close, IconButtonSize::Small, "Small")];
                SheetGrid::row(ui, "Size", Self::COLUMN, height, sizes.len(), |ui, index, cell| {
                    let (icon, size, legend) = sizes[index];
                    let side = size.side();
                    let rect = SheetGrid::place(cell, vec2(side, side));
                    IconButton::new(icon).size(size).paint(ui.painter(), rect, InteractionState::Rest);
                    let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
                    let faint = <color::TextFaint as ColorToken>::VALUE;
                    let galley = Draw::galley(ui.painter(), legend, style, faint, TextFit::Line);
                    let at = egui::pos2(rect.right() + Self::LEGEND_GAP, cell.center().y - galley.size().y / 2.0);
                    ui.painter().galley(at, galley, faint);
                });
            });
        });
    }
}

use egui::{Color32, Rect, Ui, pos2, vec2};

use crate::gallery::{GalleryState, SheetGrid};
use crate::glyph::{Glyph, KeyAuto, KeyCustom, KeyEased, KeyHold, KeyLook};
use crate::tokens::{ColorToken, color};

/// Paper: Assets, `Glyph / key / …` (`30A-0`, `35Y-0`, `36M-0`, `37A-0`). Each key shape at rest and selected, in
/// five colours. Paper uses the five track colours, which are studio tokens; this sheet uses five amml-ui role
/// colours in their place.
pub struct KeyGlyphsSheet;

impl KeyGlyphsSheet {
    /// Sheet chrome: Paper puts the keys 24 apart.
    const PITCH: f32 = 24.0;
    /// Sheet chrome: the row height, room for a selected key's halo.
    const HEIGHT: f32 = 28.0;
    const COLOURS: usize = 5;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let column = Self::PITCH * Self::COLOURS as f32;
        SheetGrid::page(ui, "Key glyphs", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, column, &["REST", "SELECTED"]);
                for (label, shape) in [("Hold", 0), ("Auto", 1), ("Eased", 2), ("Custom", 3)] {
                    SheetGrid::row(ui, label, column, Self::HEIGHT, 2, |ui, index, cell| {
                        let look = if index == 0 { KeyLook::Rest } else { KeyLook::Selected };
                        Self::keys(ui, cell, shape, look);
                    });
                }
            });
        });
    }

    // one key of `shape` in each colour, `PITCH` apart
    fn keys(ui: &Ui, cell: Rect, shape: usize, look: KeyLook) {
        let colours: [Color32; 5] = [
            <color::Accent as ColorToken>::VALUE,
            <color::Success as ColorToken>::VALUE,
            <color::Warning as ColorToken>::VALUE,
            <color::Danger as ColorToken>::VALUE,
            <color::TextDim as ColorToken>::VALUE,
        ];
        let glyph: &dyn Glyph = match shape {
            0 => &KeyHold { look },
            1 => &KeyAuto { look },
            2 => &KeyEased { look },
            _ => &KeyCustom { look },
        };
        for (index, colour) in colours.into_iter().enumerate() {
            let left = cell.left() + Self::PITCH * index as f32;
            let slot = Rect::from_min_size(pos2(left, cell.top()), vec2(Self::PITCH, cell.height()));
            glyph.paint(ui.painter(), slot, colour);
        }
    }
}

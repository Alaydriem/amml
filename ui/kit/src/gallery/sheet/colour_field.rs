use egui::{Id, Rect, Sense, Ui, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{ColourField, ColourValue, FieldLook};

/// Paper: Mod manager, `Widget / Colour field` (`DK3-0`). One cell per state, each with its caption.
pub struct ColourFieldSheet;

impl ColourFieldSheet {
    const COLUMN: f32 = 194.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let alpha = ColourValue { rgba: [0xFF, 0xFF, 0xFF, 0x40], alpha: true };
        let green = ColourValue { rgba: [0x4F, 0xD1, 0xA5, 0xFF], alpha: false };
        // (state, value, look, caption); the invalid cell's caption is the field's own reason
        let cells = [
            (InteractionState::Rest, alpha, FieldLook::default(), Some("Rest")),
            (InteractionState::Hover, alpha, FieldLook::default(), Some("Hover")),
            (InteractionState::Focused, green, FieldLook { selection: Some(7..7), invalid: None }, Some("Editing")),
            (
                InteractionState::Rest,
                alpha,
                FieldLook {
                    selection: None,
                    invalid: Some(amml_i18n::text::Text::message(crate::AmmlUiMessage::InvalidColour)),
                },
                None,
            ),
            (InteractionState::Disabled, alpha, FieldLook::default(), Some("Disabled · Turn on Chunk grid first")),
        ];
        SheetGrid::page(ui, "Colour field", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::section(ui, "STATES", |ui| {
                    let height = <spacing::Button as SpacingToken>::VALUE;
                    let caption_gap = <spacing::S as SpacingToken>::VALUE;
                    let row = height + caption_gap + TypeSize::Caption.line_height();
                    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), row), Sense::hover());
                    for (index, (state, value, look, caption)) in cells.into_iter().enumerate() {
                        let left = area.left() + index as f32 * (Self::COLUMN + SheetGrid::COLUMN_GAP);
                        let mut value = value;
                        let field = ColourField::new(Id::NULL, &mut value);
                        let width = field.desired_width(ui.ctx());
                        let rect = Rect::from_min_size(pos2(left, area.top()), vec2(width, height));
                        field.paint(ui.painter(), rect, state, &look);
                        if let Some(caption) = caption {
                            let painter = state.painter(ui.painter());
                            let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
                            let faint = <color::TextFaint as ColorToken>::VALUE;
                            let galley = Draw::galley(&painter, caption, style, faint, TextFit::Line);
                            painter.galley(pos2(left, rect.bottom() + caption_gap), galley, faint);
                        }
                    }
                });
            });
        });
    }
}

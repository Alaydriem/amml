use std::ops::Range;
use std::sync::Arc;

use amml_i18n::text::Text;
use egui::text::CCursor;
use egui::{Color32, Galley, Painter, Pos2, Rect, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// The painting the text-entry fields share: the box, the typed text with its selection and caret,
/// and the reason line under an invalid field.
pub(crate) struct FieldBox;

impl FieldBox {
    /// Paper: Text field and Drag field sheets, the caret width.
    const CARET_WIDTH: f32 = 1.0;

    pub fn paint(painter: &Painter, rect: Rect, fill: Color32, border: Option<Color32>, focused: bool) {
        if focused {
            FocusRing::paint(painter, rect, Corner::Control.radius(rect));
        }
        Draw::fill(painter, rect, Corner::Control, fill);
        if let Some(border) = border {
            Draw::stroke(painter, rect, Corner::Control, border);
        }
    }

    /// Paints `galley` at `at` with the selection fill behind `selection` (in characters) and the
    /// caret after it.
    pub fn paint_text(
        painter: &Painter,
        at: Pos2,
        galley: Arc<Galley>,
        color: Color32,
        selection: Option<Range<usize>>,
    ) {
        if let Some(range) = selection {
            let start = galley.pos_from_cursor(CCursor::new(range.start)).translate(at.to_vec2());
            let end = galley.pos_from_cursor(CCursor::new(range.end)).translate(at.to_vec2());
            if range.end > range.start {
                let selected = Rect::from_min_max(pos2(start.left(), start.top()), pos2(end.left(), end.bottom()));
                Draw::fill(painter, selected, Corner::Small, <color::Selection as ColorToken>::VALUE);
            }
            let height = <spacing::L as SpacingToken>::VALUE;
            let caret = Rect::from_center_size(pos2(end.left(), end.center().y), vec2(Self::CARET_WIDTH, height));
            painter.rect_filled(caret, 0, <color::Caret as ColorToken>::VALUE);
        }
        painter.galley(at, galley, color);
    }

    /// Paints `reason` on one line under `field`, `gap` below it; it may run past the field's right
    /// edge, as the sheets draw it. Returns the reason's rect.
    pub fn paint_reason(
        painter: &Painter,
        field: Rect,
        gap: f32,
        reason: &Text,
        style: TextStyle,
        color: Color32,
    ) -> Rect {
        let text = UiText::resolve(painter.ctx(), reason);
        let galley = Draw::galley(painter, &text, style, color, TextFit::Line);
        let at = pos2(field.left(), field.bottom() + gap);
        let rect = Rect::from_min_size(at, galley.size());
        painter.galley(at, galley, color);
        rect
    }
}

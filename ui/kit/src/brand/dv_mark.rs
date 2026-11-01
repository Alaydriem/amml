use egui::epaint::CornerRadius;
use egui::text::{LayoutJob, TextFormat};
use egui::{Painter, Rect, Stroke, StrokeKind, vec2};

use super::DvMarkSize;
use crate::font::FontFace;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Assets, `Brand / DV mark`. A flat accent square with a line-overlay rim and "DV" on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DvMark {
    pub size: DvMarkSize,
}

impl DvMark {
    /// The brand mark. It is a name, not a sentence, so it is not translated.
    pub const MARK: &'static str = "DV";

    /// Paints the mark at its own side, centred on `rect`, and returns the square it painted.
    pub fn paint(&self, painter: &Painter, rect: Rect) -> Rect {
        let side = self.size.side();
        let square = Rect::from_center_size(rect.center(), vec2(side, side));
        let rim = Stroke::new(<spacing::Border as SpacingToken>::VALUE, <color::LineOverlay as ColorToken>::VALUE);
        let fill = <color::Accent as ColorToken>::VALUE;
        painter.rect(square, CornerRadius::from(self.size.radius()), fill, rim, StrokeKind::Inside);

        let ink = <color::OnAccent as ColorToken>::VALUE;
        let format = TextFormat {
            font_id: FontFace::Semibold.id(self.size.text_points()),
            color: ink,
            extra_letter_spacing: self.size.tracking(),
            ..Default::default()
        };
        let galley = painter.layout_job(LayoutJob::single_section(Self::MARK.to_owned(), format));
        let at = square.center() - galley.size() / 2.0;
        painter.galley(at, galley, ink);
        square
    }
}

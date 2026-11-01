use egui::{Color32, Painter, Rect, Vec2, vec2};

use crate::glyph::{Glyph, Spinner, SpinnerSize};
use crate::icon::{Icon, IconSize};
use crate::motion::Motion;

/// What leads a text button's label (Paper: Text button sheet, row "Leading").
#[derive(Clone, Copy, Default)]
pub enum Leading<'a> {
    #[default]
    None,
    Icon(Icon),
    /// A painted glyph in its own colour, such as a key shape in its track colour.
    Glyph(&'a dyn Glyph, Color32),
    /// The small spinner, turning while the work runs.
    Spinner,
}

impl Leading<'_> {
    /// The square the leading part takes, or zero.
    pub fn size(&self) -> Vec2 {
        let side = match self {
            Self::None => 0.0,
            Self::Icon(_) => IconSize::Regular.points(),
            Self::Glyph(..) => IconSize::Small.points(),
            Self::Spinner => SpinnerSize::Small.side(),
        };
        vec2(side, side)
    }

    /// Paints the leading part into `rect` in the label `color` (a glyph keeps its own colour).
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        match self {
            Self::None => {}
            Self::Icon(icon) => icon.paint(painter, rect, IconSize::Regular, color),
            Self::Glyph(glyph, own) => glyph.paint(painter, rect, *own),
            Self::Spinner => {
                let turn = Motion::turn(painter.ctx());
                Spinner { size: SpinnerSize::Small, turn }.paint(painter, rect, color);
            }
        }
    }
}

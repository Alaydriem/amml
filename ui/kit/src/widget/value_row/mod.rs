mod value_row_size;

pub use value_row_size::ValueRowSize;

use amml_i18n::text::Text;
use egui::{Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Value row`. A read-only fact: the label at the left in `--color-text-dim`, the
/// value at the right in `--color-text`. It takes the full width the caller gives it.
pub struct ValueRow {
    label: Text,
    value: Text,
    size: ValueRowSize,
    words: bool,
}

impl ValueRow {
    const FIGURE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const WORDS: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(label: Text, value: Text) -> Self {
        Self { label, value, size: ValueRowSize::Large, words: false }
    }

    pub fn size(mut self, size: ValueRowSize) -> Self {
        self.size = size;
        self
    }

    /// The value is a phrase, not a figure: note size in place of caption size.
    pub fn words(mut self) -> Self {
        self.words = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), self.size.height()), Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }

    /// Paints the row in `rect`. The label keeps its room; a long value is cut to what is left.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let gap = <spacing::S as SpacingToken>::VALUE;
        let value_style = if self.words { Self::WORDS } else { Self::FIGURE };
        let label_text = UiText::resolve(painter.ctx(), &self.label);
        let value_text = UiText::resolve(painter.ctx(), &self.value);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let text = <color::Text as ColorToken>::VALUE;
        let label = Draw::galley(painter, &label_text, self.size.label(), dim, TextFit::Truncate(rect.width()));
        let room = (rect.width() - label.size().x - gap).max(0.0);
        let value = Draw::galley(painter, &value_text, value_style, text, TextFit::Truncate(room));
        let centre = rect.center().y;
        painter.galley(pos2(rect.left(), centre - label.size().y / 2.0), label, dim);
        painter.galley(pos2(rect.right() - value.size().x, centre - value.size().y / 2.0), value, text);
    }
}

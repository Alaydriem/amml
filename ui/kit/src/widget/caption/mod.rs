use std::sync::Arc;

use amml_i18n::text::Text;
use egui::text::Galley;
use egui::{Painter, Pos2, Rect, Response, Sense, Ui, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Caption`. A small faint paragraph: `--text-caption` in
/// `--color-text-faint`, wrapped to the width it is given. It is a hint under a control or a
/// panel, not a label.
pub struct Caption {
    text: Text,
}

impl Caption {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(text: Text) -> Self {
        Self { text }
    }

    /// Takes the full width left in `ui` and as many lines as the text needs.
    pub fn show(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        let galley = self.galley(ui.painter(), width);
        let (rect, response) = ui.allocate_exact_size(vec2(width, galley.size().y), Sense::hover());
        ui.painter().galley(rect.min, galley, <color::TextFaint as ColorToken>::VALUE);
        response
    }

    /// Paints the text with its top left at `top_left`, wrapped to `width`. Returns its rect.
    pub fn paint(&self, painter: &Painter, top_left: Pos2, width: f32) -> Rect {
        let galley = self.galley(painter, width);
        let rect = Rect::from_min_size(top_left, galley.size());
        painter.galley(top_left, galley, <color::TextFaint as ColorToken>::VALUE);
        rect
    }

    fn galley(&self, painter: &Painter, width: f32) -> Arc<Galley> {
        let text = UiText::resolve(painter.ctx(), &self.text);
        Draw::galley(painter, &text, Self::STYLE, <color::TextFaint as ColorToken>::VALUE, TextFit::Wrap(width))
    }
}

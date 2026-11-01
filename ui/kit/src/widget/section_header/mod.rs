use amml_i18n::text::Text;
use egui::{Painter, Rangef, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `View part / Section header`. The title of one section of a panel: a
/// `--spacing-s` top space, then a `--spacing-row` band with the tracked caption title. A later
/// section has a divider at its top, edge to edge of the panel.
pub struct SectionHeader {
    title: Text,
    divider: bool,
}

impl SectionHeader {
    const TITLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();

    /// `title` is written in upper case in the message itself.
    pub fn new(title: Text) -> Self {
        Self { title, divider: false }
    }

    /// A 1 px `--color-line` above the section. The first section of a panel has none.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// The header's height: the top space and the title band.
    pub fn height() -> f32 {
        <spacing::S as SpacingToken>::VALUE + <spacing::Row as SpacingToken>::VALUE
    }

    /// Takes the full width; the section's content starts under the response's rect.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::height()), Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }

    /// Paints the header into `rect`, whose sides are the panel content's sides. The divider runs
    /// `--spacing-l` past each side, to the panel's edges.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        if self.divider {
            let inset = <spacing::L as SpacingToken>::VALUE;
            let x = Rangef::new(rect.left() - inset, rect.right() + inset);
            Draw::divider(painter, x, rect.top(), <color::Line as ColorToken>::VALUE);
        }
        let band_top = rect.top() + <spacing::S as SpacingToken>::VALUE;
        let band_centre = band_top + <spacing::Row as SpacingToken>::VALUE / 2.0;
        let colour = <color::TextFaint as ColorToken>::VALUE;
        let text = UiText::resolve(painter.ctx(), &self.title);
        let galley = Draw::galley(painter, &text, Self::TITLE, colour, TextFit::Truncate(rect.width()));
        painter.galley(pos2(rect.left(), band_centre - galley.size().y / 2.0), galley, colour);
    }
}

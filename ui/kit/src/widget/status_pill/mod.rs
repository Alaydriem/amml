mod pill_tone;

pub use pill_tone::PillTone;

use amml_i18n::text::Text;
use egui::{Align2, Context, Painter, Pos2, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Widget / Status pill`. A dot and a word on a soft fill; one look for every
/// status, the tone picks the colour. Not interactive.
pub struct StatusPill {
    tone: PillTone,
    text: Text,
}

impl StatusPill {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold);

    pub fn new(tone: PillTone, text: Text) -> Self {
        Self { tone, text }
    }

    pub fn desired_width(&self, ctx: &Context) -> f32 {
        let label = Draw::measure(ctx, &UiText::resolve(ctx, &self.text), Self::STYLE).x;
        (Self::padding() * 2.0 + Self::dot() + Self::gap() + label).ceil()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = vec2(self.desired_width(ui.ctx()), Self::height());
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        self.paint(ui.painter(), rect.left_center(), Align2::LEFT_CENTER);
        response
    }

    /// Paints the pill placed at `anchor` by `align` (for example `RIGHT_CENTER` to end at the
    /// anchor). Returns its rect.
    pub fn paint(&self, painter: &Painter, anchor: Pos2, align: Align2) -> Rect {
        let size = vec2(self.desired_width(painter.ctx()), Self::height());
        let rect = align.anchor_size(anchor, size);
        Draw::fill(painter, rect, Corner::Full, self.tone.fill());
        let dot_centre = pos2(rect.left() + Self::padding() + Self::dot() / 2.0, rect.center().y);
        painter.circle_filled(dot_centre, Self::dot() / 2.0, self.tone.dot());
        let label = UiText::resolve(painter.ctx(), &self.text);
        let galley = Draw::galley(painter, &label, Self::STYLE, self.tone.text(), TextFit::Line);
        let x = rect.left() + Self::padding() + Self::dot() + Self::gap();
        painter.galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley, self.tone.text());
        rect
    }

    fn padding() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }

    fn gap() -> f32 {
        <spacing::Xs as SpacingToken>::VALUE
    }

    fn dot() -> f32 {
        <spacing::Dot as SpacingToken>::VALUE
    }

    fn height() -> f32 {
        <spacing::ControlSmall as SpacingToken>::VALUE
    }
}

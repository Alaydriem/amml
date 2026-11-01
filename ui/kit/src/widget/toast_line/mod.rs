use amml_i18n::text::Text;
use egui::{Context, CornerRadius, Painter, Pos2, Rect, Response, Sense, Ui, Vec2, pos2, vec2};

use super::ToastTone;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Toast · one line`. A floating pill with one line of text, a stripe in the tone colour
/// at its left edge, no icon and no close button. It is as wide as its text.
pub struct ToastLine {
    tone: ToastTone,
    text: Text,
}

impl ToastLine {
    /// Paper: Toast · one line sheet, the stripe width.
    const STRIPE: f32 = 3.0;
    /// Paper: Toast · one line sheet, the stripe's corner.
    const STRIPE_CORNER: u8 = 1;
    /// Paper: Toast · one line sheet, the padding above and below the text.
    const PAD_Y: f32 = 9.0;
    /// Paper: Toast · one line sheet, the padding left and right of the text.
    const PAD_X: f32 = 18.0;
    /// Paper: Toast · one line sheet, how much of the tone colour the border shows.
    const BORDER_MIX: f32 = 0.8;
    const STYLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);

    pub fn new(tone: ToastTone, text: Text) -> Self {
        Self { tone, text }
    }

    pub fn desired_size(&self, ctx: &Context) -> Vec2 {
        let border = <spacing::Border as SpacingToken>::VALUE;
        let words = Draw::measure(ctx, &UiText::resolve(ctx, &self.text), Self::STYLE);
        vec2(
            words.x.ceil() + 2.0 * (Self::PAD_X + border),
            Self::STYLE.size.line_height() + 2.0 * (Self::PAD_Y + border),
        )
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size(ui.ctx()), Sense::hover());
        self.paint(ui.painter(), rect.min);
        response
    }

    /// Paints the toast with its top left at `top_left`. Returns its rect.
    pub fn paint(&self, painter: &Painter, top_left: Pos2) -> Rect {
        let rect = Rect::from_min_size(top_left, self.desired_size(painter.ctx()));
        let tone = self.tone.color();
        let border = tone.gamma_multiply(Self::BORDER_MIX);
        Draw::surface(
            painter,
            rect,
            Corner::Panel,
            <color::Toast as ColorToken>::VALUE,
            Some(border),
            Elevation::Floating,
        );
        // the stripe starts inside the border and runs over it at the top and bottom
        let edge = <spacing::Border as SpacingToken>::VALUE;
        let stripe = Rect::from_min_size(pos2(rect.left() + edge, rect.top()), vec2(Self::STRIPE, rect.height()));
        painter.rect_filled(stripe, CornerRadius::same(Self::STRIPE_CORNER), tone);
        let resolved = UiText::resolve(painter.ctx(), &self.text);
        let text = <color::Text as ColorToken>::VALUE;
        let galley = Draw::galley(painter, &resolved, Self::STYLE, text, TextFit::Line);
        let at = pos2(rect.left() + edge + Self::PAD_X, rect.center().y - galley.size().y / 2.0);
        painter.galley(at, galley, text);
        rect
    }
}

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Color32, Painter, Rect, pos2};

/// Paper: Editor components, `Viewer chips and overlays` (`3TE-0`), sections A and B. A chip over the game picture:
/// a `--color-pill` pill, `--spacing-row` high, with an optional dot, and its words set off by middle dots.
pub(crate) struct ViewerPill<'a> {
    pub dot: Option<Color32>,
    pub border: Color32,
    pub parts: &'a [Text],
}

impl ViewerPill<'_> {
    pub(crate) const HEIGHT: f32 = <spacing::Row as SpacingToken>::VALUE;
    // Paper: 13 each side; with a dot, 12 before the 8 px dot and 14 after the words
    const PADDING: f32 = 13.0;
    const DOT_PADDING: f32 = 12.0;
    const END_PADDING: f32 = 14.0;
    const DOT: f32 = 8.0;
    // Paper sets the words apart with "  ·  "; code draws a 2 px dot with this gap each side
    const SEPARATOR_GAP: f32 = 7.0;
    const SEPARATOR_DOT: f32 = 1.0;
    const STYLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();

    /// The width the pill needs for all its words.
    pub(crate) fn width(&self, ctx: &egui::Context) -> f32 {
        let words: f32 =
            self.parts.iter().map(|part| Draw::measure(ctx, &UiText::resolve(ctx, part), Self::STYLE).x).sum();
        let separators = self.parts.len().saturating_sub(1) as f32 * Self::SEPARATOR_GAP * 2.0;
        self.lead() + words + separators + self.end()
    }

    /// Paints the pill filling `rect`. Words that do not fit are cut, and the words after them are left out.
    pub(crate) fn paint(&self, painter: &Painter, rect: Rect) {
        Draw::fill(painter, rect, Corner::Full, <color::Pill as ColorToken>::VALUE);
        Draw::stroke(painter, rect, Corner::Full, self.border);
        let y = rect.center().y;
        if let Some(dot) = self.dot {
            painter.circle_filled(pos2(rect.left() + Self::DOT_PADDING + Self::DOT / 2.0, y), Self::DOT / 2.0, dot);
        }
        let ink = <color::Text as ColorToken>::VALUE;
        let right = rect.right() - self.end();
        let mut x = rect.left() + self.lead();
        for (index, part) in self.parts.iter().enumerate() {
            if index > 0 {
                if x + Self::SEPARATOR_GAP * 2.0 >= right {
                    break;
                }
                let dot = pos2(x + Self::SEPARATOR_GAP, y);
                painter.circle_filled(dot, Self::SEPARATOR_DOT, <color::TextDim as ColorToken>::VALUE);
                x += Self::SEPARATOR_GAP * 2.0;
            }
            let room = right - x;
            if room <= 0.0 {
                break;
            }
            let text = UiText::resolve(painter.ctx(), part);
            let galley = Draw::galley(painter, &text, Self::STYLE, ink, TextFit::Truncate(room));
            let at = Align2::LEFT_CENTER.anchor_size(pos2(x, y), galley.size());
            x = at.right();
            painter.galley(at.min, galley, ink);
        }
    }

    fn lead(&self) -> f32 {
        if self.dot.is_some() {
            Self::DOT_PADDING + Self::DOT + <spacing::S as SpacingToken>::VALUE
        } else {
            Self::PADDING
        }
    }

    fn end(&self) -> f32 {
        if self.dot.is_some() { Self::END_PADDING } else { Self::PADDING }
    }
}

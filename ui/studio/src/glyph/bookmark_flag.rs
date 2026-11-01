use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::Glyph;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, OpacityToken, color, opacity};
use egui::{Align2, Color32, Painter, Rect, Shape, Stroke, pos2, vec2};

/// Paper: Assets, `Glyph / bookmark flag` (`38M-0`). A flag with the bookmark name on a pole that runs to the bottom
/// of the rect: solid under the flag, then dashed. The pole is at the left edge of the rect. The caller gives the
/// colour: `--color-marker`, or white for the bookmark under the pointer.
pub struct BookmarkFlag {
    pub label: Text,
}

impl BookmarkFlag {
    /// Paper: the flag is 62 × 17, `--radius-small`.
    pub const WIDTH: f32 = 62.0;
    pub const HEIGHT: f32 = 17.0;
    // Paper: the pole is solid for 36 from the top, then dashes of 3 with gaps of 3
    const SOLID: f32 = 36.0;
    const DASH: f32 = 3.0;
    // Paper: the icon centre and the name start this far from the left of the flag
    const ICON_X: f32 = 8.0;
    const LABEL_X: f32 = 16.0;
    const LABEL_END_PADDING: f32 = 4.0;
}

impl Glyph for BookmarkFlag {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let flag = Rect::from_min_size(rect.min, vec2(Self::WIDTH.min(rect.width()), Self::HEIGHT));
        // a 1 px pole on the pixel centre from under the flag; its caps stay half a pixel inside the rect
        let x = rect.left() + 0.5;
        let solid_end = (rect.top() + Self::SOLID).min(rect.bottom() - 0.5);
        painter.line_segment([pos2(x, flag.bottom()), pos2(x, solid_end)], Stroke::new(1.0, color));
        let faint = color.gamma_multiply(<opacity::Track as OpacityToken>::VALUE);
        let dashes = Shape::dashed_line(
            &[pos2(x, solid_end), pos2(x, rect.bottom() - 0.5)],
            Stroke::new(1.0, faint),
            Self::DASH,
            Self::DASH,
        );
        painter.extend(dashes);
        Draw::fill(painter, flag, Corner::Small, color);
        let ink = <color::BgApp as ColorToken>::VALUE;
        let icon = Rect::from_center_size(pos2(flag.left() + Self::ICON_X, flag.center().y), egui::Vec2::ZERO);
        Icon::Flag.paint(painter, icon, IconSize::Small, ink);
        let style = TextStyle::new(TypeSize::Micro, FontFace::Semibold);
        let fit = TextFit::Truncate(flag.width() - Self::LABEL_X - Self::LABEL_END_PADDING);
        let galley = Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.label), style, ink, fit);
        let at = Align2::LEFT_CENTER.anchor_size(pos2(flag.left() + Self::LABEL_X, flag.center().y), galley.size());
        painter.galley(at.min, galley, ink);
    }
}

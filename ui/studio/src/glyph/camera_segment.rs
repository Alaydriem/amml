use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use egui::{Painter, Rect, Shape, Stroke, pos2, vec2};

use crate::rule::LabelTint;
use crate::tokens::color as studio;

/// Paper: Assets, `Glyph / camera segment + easing pill` (`3BY-0`). The camera lane between two keys: a bar (dashed
/// for a hold) and, when it fits, a pill with the easing name in the middle.
pub struct CameraSegment {
    pub ease: Text,
    pub hold: bool,
}

impl CameraSegment {
    /// Paper: the segment row is 18 high.
    pub const HEIGHT: f32 = 18.0;
    // Paper: the bar is 4 high at 7 from the top, at 35 %; a hold is a 1 px dashed line at 8, at 50 %
    const BAR_TOP: f32 = 7.0;
    const BAR_HEIGHT: f32 = 4.0;
    const BAR_ALPHA: u8 = 0x59;
    const HOLD_Y: f32 = 8.0;
    const HOLD_ALPHA: u8 = 0x80;
    const DASH: f32 = 3.0;
    const BAR_CORNER: u8 = 1;
    // Paper: the pill is 16 high, padding 6, a 1 px border at 40 %
    const PILL_HEIGHT: f32 = 16.0;
    const PILL_PADDING: f32 = 6.0;
    const PILL_BORDER_ALPHA: u8 = 0x66;

    /// Paints the segment across `rect`.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let hue = <studio::TrackCamera as ColorToken>::VALUE;
        if self.hold {
            let y = rect.top() + Self::HOLD_Y + 0.5;
            // half a pixel in at each end, so the line's caps stay in the rect
            let line = [pos2(rect.left() + 0.5, y), pos2(rect.right() - 0.5, y)];
            let stroke = Stroke::new(1.0, hue.gamma_multiply_u8(Self::HOLD_ALPHA));
            painter.extend(Shape::dashed_line(&line, stroke, Self::DASH, Self::DASH));
        } else {
            let bar = Rect::from_min_size(
                pos2(rect.left(), rect.top() + Self::BAR_TOP),
                vec2(rect.width(), Self::BAR_HEIGHT),
            );
            painter.rect_filled(bar, Self::BAR_CORNER, hue.gamma_multiply_u8(Self::BAR_ALPHA));
        }
        let ink = LabelTint::of(hue);
        let style = TextStyle::new(TypeSize::Micro, FontFace::Regular);
        let galley = Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.ease), style, ink, TextFit::Line);
        let width = galley.size().x + Self::PILL_PADDING * 2.0;
        if width > rect.width() {
            return;
        }
        let pill = Rect::from_center_size(rect.center(), vec2(width, Self::PILL_HEIGHT));
        Draw::fill(painter, pill, Corner::Full, <color::BgPanel as ColorToken>::VALUE);
        let border = Stroke::new(1.0, hue.gamma_multiply_u8(Self::PILL_BORDER_ALPHA));
        painter.rect_stroke(pill, Corner::Full.radius(pill), border, egui::StrokeKind::Inside);
        let text = Rect::from_center_size(pill.center(), galley.size());
        painter.galley(text.min, galley, ink);
    }
}

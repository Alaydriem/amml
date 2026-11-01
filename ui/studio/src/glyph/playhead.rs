use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::Glyph;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, Color32, Painter, Rect, Shape, Stroke, pos2, vec2};

/// Paper: Assets, `Glyph / playhead` (`37Y-0`). A head with the time in it, a pointer under the head and a line to
/// the bottom of the rect. The caller gives the colour: `--color-playhead`, or `--color-scene-camera` while the
/// scene camera drives.
pub struct Playhead {
    pub label: Text,
}

impl Playhead {
    /// Paper: the head is 58 × 18, `--radius-small`.
    pub const HEAD_WIDTH: f32 = 58.0;
    pub const HEAD_HEIGHT: f32 = 18.0;
    // Paper: the line is 2 wide over a 4 wide `--color-bg-app` shadow at 35 %
    const LINE_WIDTH: f32 = 2.0;
    const SHADOW_WIDTH: f32 = 4.0;
    const SHADOW_ALPHA: u8 = 0x59;
    // Paper: the pointer under the head is 10 wide and 5.5 deep
    const POINTER_WIDTH: f32 = 10.0;
    const POINTER_DEPTH: f32 = 5.5;
    // room left and right of the time inside the head
    const LABEL_PADDING: f32 = 4.0;
}

impl Glyph for Playhead {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let x = rect.center().x;
        let head = Rect::from_center_size(
            pos2(x, rect.top() + Self::HEAD_HEIGHT / 2.0),
            vec2(Self::HEAD_WIDTH.min(rect.width()), Self::HEAD_HEIGHT),
        );
        let shadow = <color::BgApp as ColorToken>::VALUE.gamma_multiply_u8(Self::SHADOW_ALPHA);
        let column =
            |width: f32| Rect::from_x_y_ranges(x - width / 2.0..=x + width / 2.0, head.bottom()..=rect.bottom());
        painter.rect_filled(column(Self::SHADOW_WIDTH), 0, shadow);
        painter.rect_filled(column(Self::LINE_WIDTH), 0, color);
        Draw::fill(painter, head, Corner::Small, color);
        let pointer = vec![
            pos2(x - Self::POINTER_WIDTH / 2.0, head.bottom()),
            pos2(x + Self::POINTER_WIDTH / 2.0, head.bottom()),
            pos2(x, head.bottom() + Self::POINTER_DEPTH),
        ];
        painter.add(Shape::convex_polygon(pointer, color, Stroke::NONE));
        let ink = <color::OnAccent as ColorToken>::VALUE;
        let style = TextStyle::new(TypeSize::Micro, FontFace::Regular);
        let fit = TextFit::Truncate(head.width() - Self::LABEL_PADDING * 2.0);
        let galley = Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.label), style, ink, fit);
        let at = Align2::CENTER_CENTER.anchor_size(head.center(), galley.size());
        painter.galley(at.min, galley, ink);
    }
}

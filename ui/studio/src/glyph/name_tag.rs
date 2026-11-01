use amml_i18n::text::Text;
use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use egui::{Painter, Rect, vec2};

/// Paper: Assets, `Glyph / player name tag` (`3BA-0`). A player's name on a dark tag over the world.
pub struct NameTag {
    pub name: Text,
}

impl NameTag {
    // Paper: padding 2 top and bottom, 5 left and right; a 2 px corner
    const PADDING_X: f32 = 5.0;
    const PADDING_Y: f32 = 2.0;
    const CORNER: u8 = 2;

    /// Paints the tag centred in `rect`, no wider than `rect`; a long name is cut. Returns the tag's rect.
    pub fn paint(&self, painter: &Painter, rect: Rect) -> Rect {
        let ink = <color::OnAccent as ColorToken>::VALUE;
        let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        let fit = TextFit::Truncate((rect.width() - Self::PADDING_X * 2.0).max(0.0));
        let galley = Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.name), style, ink, fit);
        let size = galley.size() + vec2(Self::PADDING_X, Self::PADDING_Y) * 2.0;
        let tag = Rect::from_center_size(rect.center(), vec2(size.x.min(rect.width()), size.y));
        painter.rect_filled(tag, Self::CORNER, <color::Dim as ColorToken>::VALUE);
        let text = Rect::from_center_size(tag.center(), galley.size());
        painter.galley(text.min, galley, ink);
        tag
    }
}

use egui::{Rect, TextureId, Vec2, pos2, vec2};

/// The picture behind the hero card: a texture and its size in pixels. The card shows it cropped to fill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeroPicture {
    pub texture: TextureId,
    pub size: Vec2,
}

impl HeroPicture {
    /// The part of the texture that fills a card of `card` size without stretching, centred (CSS `cover`).
    pub fn cover(&self, card: Vec2) -> Rect {
        let scale = (card.x / self.size.x).max(card.y / self.size.y);
        let shown = card / scale;
        let min = pos2((self.size.x - shown.x) / 2.0 / self.size.x, (self.size.y - shown.y) / 2.0 / self.size.y);
        Rect::from_min_size(min, vec2(shown.x / self.size.x, shown.y / self.size.y))
    }
}

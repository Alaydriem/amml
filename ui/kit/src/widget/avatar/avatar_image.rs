use std::sync::Arc;

use egui::epaint::Brush;
use egui::{Rect, TextureId, Vec2, pos2, vec2};

/// The picture on an avatar: a texture and its size in pixels. The avatar shows it cropped to fill.
/// The brush is built once, so a frame that paints the picture shares it instead of allocating one.
#[derive(Debug, Clone, PartialEq)]
pub struct AvatarImage {
    pub texture: TextureId,
    pub size: Vec2,
    brush: Arc<Brush>,
}

impl AvatarImage {
    pub fn new(texture: TextureId, size: Vec2) -> Self {
        let uv = Self::cover(size);
        Self { texture, size, brush: Arc::new(Brush { fill_texture_id: texture, uv }) }
    }

    /// The part of the texture that fills a square tile without stretching, centred. A texture
    /// with no size gives the whole texture.
    pub fn cover_uv(&self) -> Rect {
        self.brush.uv
    }

    /// The textured fill for the tile, shared with every frame that paints this picture.
    pub fn brush(&self) -> Arc<Brush> {
        Arc::clone(&self.brush)
    }

    fn cover(size: Vec2) -> Rect {
        let (w, h) = (size.x, size.y);
        let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        if w <= 0.0 || h <= 0.0 {
            return full;
        }
        if w > h {
            let margin = (1.0 - h / w) / 2.0;
            Rect::from_min_size(pos2(margin, 0.0), vec2(1.0 - 2.0 * margin, 1.0))
        } else if h > w {
            let margin = (1.0 - w / h) / 2.0;
            Rect::from_min_size(pos2(0.0, margin), vec2(1.0, 1.0 - 2.0 * margin))
        } else {
            full
        }
    }
}

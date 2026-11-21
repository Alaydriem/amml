use amml_abi::LocatorBarIconDraw;

use crate::{ActorUniqueId, Rect, Rgba, TextureId};

/// Typed reads and the writes of a `LocatorBarIconDraw`.
///
/// The loader applies `rect`, `color` and `texture` after the last listener. A texture that is unknown, stale or not
/// ready drops every edit of the call, and the game draws its own icon in the colour of the waypoint.
pub trait LocatorBarIconDrawExt {
    /// The actor the waypoint follows; `None` for a waypoint of another kind (a death marker, for example).
    fn actor(&self) -> Option<ActorUniqueId>;

    /// The rectangle of the icon in GUI units, as the game draws it now.
    fn rect(&self) -> Rect;

    /// The colour of the icon, as the game draws it now.
    fn color(&self) -> Rgba;

    /// Draws `texture` in place of the icon of the game. The whole texture is drawn in the rectangle; set the colour
    /// to white to draw it as it is.
    fn replace_texture(&mut self, texture: TextureId);

    /// Draws the icon in `rect`.
    fn set_rect(&mut self, rect: Rect);

    /// Draws the icon in `color`.
    fn set_color(&mut self, color: Rgba);
}

impl LocatorBarIconDrawExt for LocatorBarIconDraw {
    fn actor(&self) -> Option<ActorUniqueId> {
        (self.has_actor != 0).then_some(ActorUniqueId(self.waypoint_actor_id))
    }

    fn rect(&self) -> Rect {
        let [x, y, width, height] = self.rect;
        Rect { x, y, width, height }
    }

    fn color(&self) -> Rgba {
        let [a, r, g, b] = self.color.to_be_bytes();
        Rgba { r, g, b, a }
    }

    fn replace_texture(&mut self, texture: TextureId) {
        self.texture = texture.into();
    }

    fn set_rect(&mut self, rect: Rect) {
        self.rect = [rect.x, rect.y, rect.width, rect.height];
    }

    fn set_color(&mut self, color: Rgba) {
        self.color = u32::from_be_bytes([color.a, color.r, color.g, color.b]);
    }
}

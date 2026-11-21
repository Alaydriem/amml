use crate::{AmmlEvent, AmmlEventHeader, AmmlTextureId};

/// The game is about to draw one waypoint icon of the locator bar (`ui.locator_bar_icon`, Before the draw, not
/// cancellable). One event is raised for each visible waypoint, once for each frame.
///
/// `rect` is `x, y, width, height` and `uv` is `u0, u1, v0, v1`, as the game's icon draw takes them. `color` and
/// `second_color` are packed `0xAARRGGBB`; `color` is the colour of the waypoint (white when it has none), not the
/// marker the loader uses to find the waypoint. `texture` is 0 when the game draws its own icon.
///
/// A listener can change `rect`, `color` and `texture`; the loader writes every other field back after each
/// listener. A `texture` of 0 keeps the game's icon, and the other two changes still apply. A `texture` that is
/// unknown, stale or not ready drops every change of the call and the game draws its own icon.
///
/// Raised on the render thread, or on the game thread when the game draws there (the context says which).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LocatorBarIconDraw {
    pub header: AmmlEventHeader,
    /// The unique id of the actor the waypoint follows; read only when `has_actor` is not 0.
    pub waypoint_actor_id: i64,
    pub has_actor: u8,
    pub _reserved: [u8; 7],
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub light: f32,
    pub alpha: f32,
    pub color: u32,
    pub second_color: u32,
    pub scale: [f32; 2],
    pub texture: AmmlTextureId,
}

impl AmmlEvent for LocatorBarIconDraw {
    const ID: &'static str = "ui.locator_bar_icon";
}

const _: () = assert!(core::mem::size_of::<LocatorBarIconDraw>() == 104);

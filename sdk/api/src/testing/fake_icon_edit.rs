use crate::{Rect, Rgba, TextureId};

/// What the game draws for one locator bar icon after the listeners of `LocatorBarIconDraw` ran, as
/// `FakeGame::raise_locator_icon` gives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FakeIconEdit {
    /// The texture a listener asked for; `None` when the game draws its own icon.
    pub texture: Option<TextureId>,
    /// The rectangle of the icon.
    pub rect: Rect,
    /// The colour of the icon.
    pub color: Rgba,
}

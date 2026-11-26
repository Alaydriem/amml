/// The camera of a block entity pass (`BlockEntityPass::camera`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Zero is not a camera.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmmlBlockEntityCamera(pub u8);

impl AmmlBlockEntityCamera {
    /// The level renderer camera of the local player (its vtable is `LevelRendererPlayer.vtable`): the view the player sees.
    pub const PLAYER: Self = Self(1);
    /// Any other level renderer camera.
    pub const OTHER: Self = Self(2);
}

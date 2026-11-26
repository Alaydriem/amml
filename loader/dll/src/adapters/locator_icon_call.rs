/// The arguments of one call of the icon blit of the locator bar, after the render context. `rect` is `x, y, width,
/// height` and `uv` is `u0, u1, v0, v1`, in the order the game takes them. `texture` is the address of a `TexturePtr`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocatorIconCall {
    pub texture: usize,
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub light: f32,
    pub alpha: f32,
    pub color: u32,
    pub second_color: u32,
    pub scale: [f32; 2],
}

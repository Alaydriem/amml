/// A rectangle in GUI units, the units of the game's UI layout (not pixels): `x` and `y` are the top left corner.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlRect>() == 16);

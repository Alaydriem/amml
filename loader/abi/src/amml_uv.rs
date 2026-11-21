/// A part of a texture in normalized coordinates: the corner `(u0, v0)` and the opposite corner `(u1, v1)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlUv {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlUv>() == 16);

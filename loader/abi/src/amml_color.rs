/// A colour of three channels, each from 0.0 to 1.0. Its alpha travels apart, as a separate argument.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlColor>() == 12);

/// A colour of four 8-bit channels.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

const _: () = assert!(core::mem::size_of::<AmmlRgba>() == 4);

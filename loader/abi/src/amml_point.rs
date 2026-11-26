/// A point in pixels, in client or back buffer coordinates.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlPoint {
    pub x: f32,
    pub y: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlPoint>() == 8);

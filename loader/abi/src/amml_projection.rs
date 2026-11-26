/// The projection of the camera of a frame: the vertical field of view in degrees and the aspect ratio (width over
/// height).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AmmlProjection {
    pub fov_y: f32,
    pub aspect: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlProjection>() == 8);

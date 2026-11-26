/// The vanilla camera the player sees the world through (`CameraControl::active_camera`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Zero is not a kind.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlCameraKind(pub u8);

impl AmmlCameraKind {
    /// The `minecraft:first_person` camera.
    pub const FIRST_PERSON: Self = Self(1);
    /// The `minecraft:third_person` camera.
    pub const THIRD_PERSON: Self = Self(2);
    /// The `minecraft:third_person_front` camera.
    pub const THIRD_PERSON_FRONT: Self = Self(3);
    /// Any other active camera: a free camera, a camera a server set, or a camera of another name.
    pub const OTHER: Self = Self(4);
}

use amml_abi::AmmlProjection;

/// The projection of the camera of a frame: the vertical field of view in degrees and the aspect ratio (width over
/// height).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Projection {
    pub fov_y: f32,
    pub aspect: f32,
}

impl From<AmmlProjection> for Projection {
    fn from(projection: AmmlProjection) -> Self {
        Self { fov_y: projection.fov_y, aspect: projection.aspect }
    }
}

impl From<Projection> for AmmlProjection {
    fn from(projection: Projection) -> Self {
        Self { fov_y: projection.fov_y, aspect: projection.aspect }
    }
}

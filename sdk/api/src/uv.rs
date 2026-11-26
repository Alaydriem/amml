use amml_abi::AmmlUv;

/// A part of a texture in normalized coordinates (0.0 to 1.0): the corner `(u0, v0)` and the opposite corner
/// `(u1, v1)`. `Uv::FULL` is the whole texture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Uv {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl Uv {
    pub const FULL: Self = Self { u0: 0.0, v0: 0.0, u1: 1.0, v1: 1.0 };
}

impl From<AmmlUv> for Uv {
    fn from(uv: AmmlUv) -> Self {
        Self { u0: uv.u0, v0: uv.v0, u1: uv.u1, v1: uv.v1 }
    }
}

impl From<Uv> for AmmlUv {
    fn from(uv: Uv) -> Self {
        Self { u0: uv.u0, v0: uv.v0, u1: uv.u1, v1: uv.v1 }
    }
}

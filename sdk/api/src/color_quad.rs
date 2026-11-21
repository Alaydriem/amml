use amml_abi::AmmlColorQuad;

use crate::{Rgba, Vec3};

/// One quad of a world mesh: four corners in mesh space, in blocks, in drawing order, and one colour for the whole
/// quad. With `MeshStyle::two_sided` false only the side the corners face is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorQuad {
    pub corners: [Vec3; 4],
    pub rgba: Rgba,
}

impl From<ColorQuad> for AmmlColorQuad {
    fn from(quad: ColorQuad) -> Self {
        Self { corners: quad.corners.map(Into::into), rgba: quad.rgba.into() }
    }
}

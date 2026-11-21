use amml_abi::AmmlMeshStyle;

use crate::{Blend, MeshLight, MeshTexture};

/// How the loader builds and draws one world mesh. This version draws the `OVERLAY` style, with or without
/// `two_sided`; any other blend, depth, light or texture gives `NotAvailable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshStyle {
    pub blend: Blend,
    pub depth_test: bool,
    pub depth_write: bool,
    /// Both sides of each quad are drawn (the loader writes the quad a second time with its corners reversed).
    pub two_sided: bool,
    pub light: MeshLight,
    pub texture: MeshTexture,
}

impl MeshStyle {
    /// Alpha blend, hidden behind the world, no depth write, one side, full light, the white texture.
    pub const OVERLAY: Self = Self {
        blend: Blend::Alpha,
        depth_test: true,
        depth_write: false,
        two_sided: false,
        light: MeshLight::Full,
        texture: MeshTexture::White,
    };
}

impl From<MeshStyle> for AmmlMeshStyle {
    fn from(style: MeshStyle) -> Self {
        Self {
            blend: match style.blend {
                Blend::Opaque => AmmlMeshStyle::BLEND_OPAQUE,
                Blend::Alpha => AmmlMeshStyle::BLEND_ALPHA,
            },
            depth_test: u8::from(style.depth_test),
            depth_write: u8::from(style.depth_write),
            two_sided: u8::from(style.two_sided),
            light: match style.light {
                MeshLight::Full => AmmlMeshStyle::LIGHT_FULL,
                MeshLight::World => AmmlMeshStyle::LIGHT_WORLD,
            },
            texture: match style.texture {
                MeshTexture::White => AmmlMeshStyle::TEXTURE_WHITE,
                MeshTexture::TerrainAtlas => AmmlMeshStyle::TEXTURE_TERRAIN_ATLAS,
            },
            _reserved: [0; 2],
        }
    }
}

use amml_abi::AmmlMeshStyle;

use crate::{Blend, MeshLight, MeshTexture};

/// How the loader builds and draws one world mesh. A build from colour quads serves the `OVERLAY` style, with or
/// without `two_sided`; any other blend, depth, light or texture gives `NotAvailable`, `GHOST` included.
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
    /// What a block mesh of the block mesher is drawn with: the client's blend-block material and the terrain atlas.
    /// Alpha blend, hidden behind the world, full light, one side. The depth write is the one of the vanilla
    /// `entity_alphablend` material definition (`entity.material`: the state `Blending` only, no `DisableDepthWrite`, on
    /// 26.63 and 70.26): it writes depth. `WorldCanvas::build` from colour quads gives `NotAvailable` for it: colour
    /// quads carry no UV, so only `BlockMesher` makes such a mesh.
    pub const GHOST: Self = Self {
        blend: Blend::Alpha,
        depth_test: true,
        depth_write: true,
        two_sided: false,
        light: MeshLight::Full,
        texture: MeshTexture::TerrainAtlas,
    };

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

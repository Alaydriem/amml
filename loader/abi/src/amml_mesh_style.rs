/// How the loader builds and draws one world mesh. Each field is one byte; the reserved bytes are 0.
///
/// `blend`: `BLEND_OPAQUE` or `BLEND_ALPHA`. `depth_test`, `depth_write`, `two_sided`: 0 or 1. `light`: `LIGHT_FULL`
/// (the mesh colour as given) or `LIGHT_WORLD`. `texture`: `TEXTURE_WHITE` (the vertex colour only) or
/// `TEXTURE_TERRAIN_ATLAS`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlMeshStyle {
    pub blend: u8,
    pub depth_test: u8,
    pub depth_write: u8,
    pub two_sided: u8,
    pub light: u8,
    pub texture: u8,
    pub _reserved: [u8; 2],
}

impl AmmlMeshStyle {
    pub const BLEND_OPAQUE: u8 = 0;
    pub const BLEND_ALPHA: u8 = 1;
    pub const LIGHT_FULL: u8 = 0;
    pub const LIGHT_WORLD: u8 = 1;
    pub const TEXTURE_WHITE: u8 = 0;
    pub const TEXTURE_TERRAIN_ATLAS: u8 = 1;
}

const _: () = assert!(core::mem::size_of::<AmmlMeshStyle>() == 8);

use crate::{AmmlBlockHandle, AmmlLightFalloff, AmmlVec3};

/// One held light, as a mod gives it to `AmmlLightingTable::set_light_source`: a world position, a level
/// (1 to 15), a falloff and, when `has_block` is 1, the block the light is (for the Vibrant Visuals emitter).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlLightSource {
    pub size: u32,
    pub version: u32,
    pub position: AmmlVec3,
    pub level: u8,
    pub falloff: AmmlLightFalloff,
    pub has_block: u8,
    pub _reserved: u8,
    pub block: AmmlBlockHandle,
}

impl AmmlLightSource {
    /// `size` of a v1 light source. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 32;
}

const _: () = assert!(core::mem::size_of::<AmmlLightSource>() == AmmlLightSource::SIZE_V1 as usize);

use crate::{AmmlLightSource, AmmlStatus, AmmlStr};

/// Sets the held light of `module`; a null `source` clears it. Game thread only (`REFUSED` elsewhere,
/// `NOT_READY` before the first frame). A `source` whose `size` is below `AmmlLightSource::SIZE_V1`, whose
/// level is not 1 to 15 or whose position is not finite gives `INVALID_ARGUMENT`; a block handle of a left
/// world gives `STALE`; a fifth mod with a light gives `REFUSED`.
pub type LightingSetSourceFn = unsafe extern "C" fn(module: AmmlStr, source: *const AmmlLightSource) -> AmmlStatus;

/// The lighting functions of `AmmlServices` (areas `lighting.terrain`, `lighting.entity`,
/// `lighting.vibrant`). The table only grows: a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlLightingTable {
    pub size: u32,
    pub version: u32,
    pub set_light_source: LightingSetSourceFn,
}

impl AmmlLightingTable {
    /// `size` of a v1 lighting table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 16;
}

const _: () = assert!(core::mem::size_of::<AmmlLightingTable>() == AmmlLightingTable::SIZE_V1 as usize);

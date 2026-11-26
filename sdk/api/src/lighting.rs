use amml_abi::{AmmlBlockHandle, AmmlLightSource, AmmlLightingTable, AmmlStr, AmmlVec3};

use crate::{ApiError, LightSource};

/// The loader-owned held light (`Area::LIGHTING_TERRAIN`). The loader draws the terrain near the light
/// lit (the block light byte is raised, never lowered; the sky byte never changes), follows the light in
/// quarter-block steps and rebuilds the meshes in reach. With `Area::LIGHTING_ENTITY` entities and block
/// entities near the light are lit too; with `Area::LIGHTING_VIBRANT` a light with a `block` is a real
/// Vibrant Visuals emitter with the block's own colour. Visual only: the world's light values do not
/// change.
///
/// Each mod has one light; up to four mods can hold one, and a block gets the brightest value. The loader
/// clears a mod's light at its disable (the lit area goes dark at once) and every light at a world leave.
pub struct Lighting {
    table: Option<AmmlLightingTable>,
    module: &'static str,
}

impl Lighting {
    pub(crate) fn new(table: Option<AmmlLightingTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Sets this mod's light; `None` clears it. Game thread only (`Refused` elsewhere, `NotReady` before the
    /// first frame). A level outside 1 to 15 or a position that is not finite gives `InvalidArgument`; a
    /// block handle of a left world `Stale`; a fifth mod with a light `Refused`. A call with the same light
    /// as before (the same quarter block, level, falloff and block) changes nothing and costs no rebuild.
    pub fn set_light_source(&self, source: Option<LightSource>) -> Result<(), ApiError> {
        let Some(table) = self.table else { return Err(ApiError::NotAvailable) };
        let raw = source.map(|source| AmmlLightSource {
            size: AmmlLightSource::SIZE_V1,
            version: 1,
            position: AmmlVec3 { x: source.position.x, y: source.position.y, z: source.position.z },
            level: source.level,
            falloff: source.falloff.into(),
            has_block: u8::from(source.block.is_some()),
            _reserved: 0,
            block: source.block.map_or(AmmlBlockHandle::NONE, |block| block.raw()),
        });
        let pointer = raw.as_ref().map_or(std::ptr::null(), std::ptr::from_ref);
        // SAFETY: the module id and the light source live for the call; a null source clears.
        #[allow(unsafe_code)]
        let status = unsafe { (table.set_light_source)(AmmlStr::from_str(self.module), pointer) };
        ApiError::check(status)
    }
}

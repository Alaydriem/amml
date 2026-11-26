use amml_abi::{AmmlBlockEntityGhost, AmmlGhostEntityOptions, AmmlRenderTable, AmmlStr};

use crate::{ApiError, BlockEntityGhost, BlockHandle, GhostEntityOptions};

/// Block entity ghosts (`Area::RENDER_BLOCK_ENTITIES`): the game's own block actors of blocks (chests, signs, banners,
/// beds and the other blocks with a block entity), owned by the loader and drawn in a `BlockEntityPass` with
/// `BlockEntityPassExt::draw_block_entity`.
///
/// `create` and `free` run on the game thread with a world joined; the ghosts are bound to this mod and that world.
/// Without a render table of version 5 every call gives `Err(NotAvailable)`.
#[derive(Clone, Copy)]
pub struct BlockEntities {
    table: Option<AmmlRenderTable>,
    module: &'static str,
}

impl BlockEntities {
    pub(crate) fn new(table: Option<AmmlRenderTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Makes a ghost of `block`: its block actor loaded from `nbt` (the block entity compound in little-endian NBT, as a
    /// structure file stores it) or, with `None` or empty bytes, made with its defaults; then the `options`.
    /// `Err(Refused)` off the game thread or when this mod holds `BlockEntityGhost::MAX_LIVE` ghosts; `Err(NotReady)`
    /// without a joined world; `Err(Stale)` for a handle of an earlier world; `Err(NotAvailable)` for an option whose area
    /// is not offered, and from a loader without `Area::RENDER_BLOCK_ENTITIES`; `Err(Loader(INVALID_ARGUMENT))` for a
    /// chest pair that is not one block along x or z, or on a block that is not a chest, and for a skull pose with an
    /// unknown face or a turn above 15, or on a block that is not a skull; `Err(Loader(FAILED))` when the
    /// game made no actor (a block without a block entity, or NBT the game cannot read).
    #[allow(unsafe_code)]
    pub fn create(
        &self,
        block: BlockHandle,
        nbt: Option<&[u8]>,
        options: GhostEntityOptions,
    ) -> Result<BlockEntityGhost, ApiError> {
        let create = self.table.and_then(|table| table.block_entity_create).ok_or(ApiError::NotAvailable)?;
        let bytes = nbt.unwrap_or(&[]);
        let raw_options = AmmlGhostEntityOptions::from(&options);
        let mut out = AmmlBlockEntityGhost::default();
        // SAFETY: the table function is the loader's; the module id, the bytes and the options live for the call; `out`
        // is writable. Empty bytes go as a null pointer with a length of 0.
        let status = unsafe {
            create(
                AmmlStr::from_str(self.module),
                block.raw(),
                if bytes.is_empty() { std::ptr::null() } else { bytes.as_ptr() },
                bytes.len(),
                &raw const raw_options,
                &raw mut out,
            )
        };
        ApiError::check(status)?;
        Ok(BlockEntityGhost::from_raw(out))
    }

    /// Frees `ghost`. Its game actor is released by the loader on the render thread. A ghost the loader already freed
    /// (world leave, disable, device reset) is ignored.
    #[allow(unsafe_code)]
    pub fn free(&self, ghost: BlockEntityGhost) {
        let Some(free) = self.table.and_then(|table| table.block_entity_free) else { return };
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let _ = unsafe { free(AmmlStr::from_str(self.module), ghost.raw()) };
    }
}

use amml_abi::{AmmlBlockPos, AmmlStr, BlockEntityPass};

use crate::lent_render::LentRender;
use crate::{ApiError, BlockEntityCamera, BlockEntityGhost, BlockPos};

/// Typed reads and the draw of a `BlockEntityPass`: the game drew the opaque block entities of one level renderer
/// camera, and the event comes after that pass. Raised only while a world is joined, on the game thread.
pub trait BlockEntityPassExt {
    /// The camera of this pass: `Player` for the view the player sees.
    fn camera(&self) -> BlockEntityCamera;

    /// Draws `ghost` at the world position `pos` with the game's block actor renderer, in this pass: opaque, untinted,
    /// with the camera of the pass. Only for the event of a running listener call on this thread: a copy, or one kept
    /// past its call, gives `Err(Refused)`. `Err(Stale)` for a ghost the loader freed; `Err(NotAvailable)` from a loader
    /// without `Area::RENDER_BLOCK_ENTITIES`; `Err(Loader(FAILED))` when the actor has no renderer.
    fn draw_block_entity(&self, ghost: &BlockEntityGhost, pos: BlockPos) -> Result<(), ApiError>;
}

impl BlockEntityPassExt for BlockEntityPass {
    fn camera(&self) -> BlockEntityCamera {
        BlockEntityCamera::from(self.camera)
    }

    #[allow(unsafe_code)]
    fn draw_block_entity(&self, ghost: &BlockEntityGhost, pos: BlockPos) -> Result<(), ApiError> {
        let lent = LentRender::find(std::ptr::from_ref(self)).ok_or(ApiError::Refused)?;
        let draw = lent.table.block_entity_draw.ok_or(ApiError::NotAvailable)?;
        let at = AmmlBlockPos::from(pos);
        // SAFETY: the table function is the loader's; the module id and the position live for the call.
        let status = unsafe { draw(AmmlStr::from_str(lent.module), self.frame_id, ghost.raw(), &raw const at) };
        ApiError::check(status)
    }
}

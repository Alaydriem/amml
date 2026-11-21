use amml_abi::{AmmlApplyStats, AmmlShapeOp, AmmlStr, AmmlWorldTable};

use crate::{ApiError, ApplyStats, ShapeOp};

/// Lines, boxes and arrows the game draws in the world (`Ctx::world_shapes`), the way it draws a server's script
/// shapes. `Area::WORLD_SHAPES`.
///
/// The shapes stay until this mod removes them, leaves the world, or the loader sees a new client packet handler: then
/// `target_epoch` changes, the game has dropped every shape, and the mod sends its shapes again. At this mod's disable
/// the loader removes its shapes. Game thread only for `apply`.
pub struct WorldShapes {
    table: Option<AmmlWorldTable>,
    module: &'static str,
}

impl WorldShapes {
    pub(crate) fn new(table: Option<AmmlWorldTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Hands `ops` to the game in one packet, in order, as shapes of the dimension `dimension` (the id of
    /// `Dimension::id`). `Err(NotReady)` while no world is joined, and before the loader saw the client's packet
    /// handler in this world (at the next server packet of a hooked handler slot, such as the chunk publisher update);
    /// `Err(Refused)` for more than `max_batch` ops, off the game thread, or from a listener of a free camera call's
    /// `CameraInstructionReceived`; `Err(Loader(INVALID_ARGUMENT))` for a number that is not finite (nothing goes out);
    /// `Err(Loader(FAILED))` when the game made no packet; `Err(NotAvailable)` without the area.
    #[allow(unsafe_code)]
    pub fn apply(&self, dimension: i32, ops: &[ShapeOp]) -> Result<ApplyStats, ApiError> {
        let Some(apply) = self.table.and_then(|table| table.shapes_apply) else { return Err(ApiError::NotAvailable) };
        let ops: Vec<AmmlShapeOp> = ops.iter().copied().map(AmmlShapeOp::from).collect();
        let count = u32::try_from(ops.len()).map_err(|_| ApiError::Refused)?;
        let mut stats = AmmlApplyStats::default();
        // SAFETY: the table function is the loader's; the module id, the ops and the stats live for the call.
        let status = unsafe { apply(AmmlStr::from_str(self.module), dimension, ops.as_ptr(), count, &raw mut stats) };
        ApiError::check(status).map(|()| ApplyStats::from(stats))
    }

    /// The most ops one `apply` takes (800 on this loader); 0 without the area.
    #[allow(unsafe_code)]
    pub fn max_batch(&self) -> usize {
        let Some(max_batch) = self.table.and_then(|table| table.shapes_max_batch) else { return 0 };
        // SAFETY: the table function is the loader's and takes no arguments.
        usize::try_from(unsafe { max_batch() }).unwrap_or(0)
    }

    /// The generation of the client packet handler the loader holds. When it differs from the value at your last
    /// `apply`, the game dropped your shapes: send them again. `None` before the loader saw the handler in this world,
    /// and without the area. Any thread.
    #[allow(unsafe_code)]
    pub fn target_epoch(&self) -> Option<u32> {
        let target_epoch = self.table.and_then(|table| table.shapes_target_epoch)?;
        let mut out = 0u32;
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { target_epoch(&raw mut out) };
        status.is_ok().then_some(out)
    }
}

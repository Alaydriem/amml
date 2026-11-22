use amml_abi::WorldRendered;

use crate::lent_render::LentRender;
use crate::{BlockMesher, WorldCanvas};

/// Typed reads of a `WorldRendered`: the game drew the world of the primary client, and the event comes after its level
/// pass.
///
/// The event is raised only while a world is joined, on the thread that runs the game's `renderLevel`: the game thread
/// or the render thread (`EventCtx::thread`).
pub trait WorldRenderedExt {
    /// The id of this dispatch. It is new for each frame.
    fn frame_id(&self) -> u64;

    /// Whether the game renders in the Vibrant Visuals (deferred) graphics mode. The loader reads the mode at each client
    /// tick, so a switch shows from the first frame after the tick that saw it.
    fn vibrant(&self) -> bool;

    /// The canvas of this frame: `None` when the loader has no render table or does not offer the area
    /// `Area::RENDER_WORLD_CANVAS`, and for any `WorldRendered` that is not the event of a running listener call on
    /// this thread (a copy, or one kept past its call).
    fn canvas(&self) -> Option<WorldCanvas<'_>>;

    /// The block mesher of this frame: `None` when the loader's render table has no mesher (older than version 4), and,
    /// as for `canvas`, for any `WorldRendered` that is not the event of a running listener call on this thread. A loader
    /// that does not offer `Area::RENDER_BLOCK_MESHER` answers each build with `Err(NotAvailable)`.
    fn mesher(&self) -> Option<BlockMesher<'_>>;
}

impl WorldRenderedExt for WorldRendered {
    fn frame_id(&self) -> u64 {
        self.frame_id
    }

    fn vibrant(&self) -> bool {
        self.vibrant != 0
    }

    fn canvas(&self) -> Option<WorldCanvas<'_>> {
        let lent = LentRender::find(std::ptr::from_ref(self))?;
        Some(WorldCanvas::new(lent.table, self.frame_id, lent.module))
    }

    fn mesher(&self) -> Option<BlockMesher<'_>> {
        let lent = LentRender::find(std::ptr::from_ref(self))?;
        lent.table.mesher_build?;
        Some(BlockMesher::new(lent.table, self.frame_id, lent.module))
    }
}

use std::cell::Cell;

use crate::RenderFrameGuard;

thread_local! {
    // The frame id of the `WorldRendered` dispatch running on this thread; 0 outside a dispatch.
    static CURRENT: Cell<u64> = const { Cell::new(0) };
}

/// The frame of the `WorldRendered` dispatch that runs on the calling thread. A canvas call names a frame id and is
/// allowed only while that frame is the current one on its own thread; the id 0 is never a frame.
pub struct RenderScope;

impl RenderScope {
    /// Makes `frame` the current frame of this thread until the guard is dropped.
    pub fn enter(frame: u64) -> RenderFrameGuard {
        let previous = CURRENT.with(|current| current.replace(frame));
        RenderFrameGuard { previous }
    }

    /// The frame of the dispatch running on this thread.
    pub fn current() -> Option<u64> {
        Some(CURRENT.with(Cell::get)).filter(|frame| *frame != 0)
    }

    /// Whether `frame` is the frame of the dispatch running on this thread.
    pub fn allows(frame: u64) -> bool {
        frame != 0 && Self::current() == Some(frame)
    }

    pub(crate) fn restore(previous: u64) {
        CURRENT.with(|current| current.set(previous));
    }
}

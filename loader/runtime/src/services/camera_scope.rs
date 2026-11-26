use std::cell::Cell;

use crate::CameraFrameGuard;

thread_local! {
    // The frame id of the `CameraSetup` dispatch running on this thread; 0 outside a dispatch.
    static CURRENT: Cell<u64> = const { Cell::new(0) };
}

/// The frame of the `CameraSetup` dispatch that runs on the calling thread. A camera call names a frame id and is
/// allowed only while that frame is the current one on its own thread; the id 0 is never a frame. It is separate from
/// `RenderScope`, so a camera call is refused in a `WorldRendered` dispatch and a canvas call in a `CameraSetup` one.
pub struct CameraScope;

impl CameraScope {
    /// Makes `frame` the current camera frame of this thread until the guard is dropped.
    pub fn enter(frame: u64) -> CameraFrameGuard {
        let previous = CURRENT.with(|current| current.replace(frame));
        CameraFrameGuard { previous }
    }

    /// The frame of the camera dispatch running on this thread.
    pub fn current() -> Option<u64> {
        Some(CURRENT.with(Cell::get)).filter(|frame| *frame != 0)
    }

    /// Whether `frame` is the frame of the camera dispatch running on this thread.
    pub fn allows(frame: u64) -> bool {
        frame != 0 && Self::current() == Some(frame)
    }

    pub(crate) fn restore(previous: u64) {
        CURRENT.with(|current| current.set(previous));
    }
}

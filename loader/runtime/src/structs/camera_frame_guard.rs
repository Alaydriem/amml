use crate::CameraScope;

/// The frame of a running `CameraSetup` dispatch on this thread, until it is dropped; the frame that was current
/// before comes back then.
#[must_use = "the frame ends when the guard is dropped"]
pub struct CameraFrameGuard {
    pub(crate) previous: u64,
}

impl Drop for CameraFrameGuard {
    fn drop(&mut self) {
        CameraScope::restore(self.previous);
    }
}

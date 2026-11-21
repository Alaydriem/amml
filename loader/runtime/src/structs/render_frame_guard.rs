use crate::RenderScope;

/// The frame of a running `WorldRendered` dispatch on this thread, until it is dropped; the frame that was current
/// before comes back then.
#[must_use = "the frame ends when the guard is dropped"]
pub struct RenderFrameGuard {
    pub(crate) previous: u64,
}

impl Drop for RenderFrameGuard {
    fn drop(&mut self) {
        RenderScope::restore(self.previous);
    }
}

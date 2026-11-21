use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{ItemDrawAdapter, TextAdapter, UiDrawAdapter};

/// The end of one `ScreenRender` dispatch with render context `context`: when it drops, the images the listeners
/// queued are drawn, the queued text is drawn with its font frame closed, and the item draw context is destroyed.
/// It runs on unwind too, so a panic inside the dispatch cannot leak a frame.
pub struct DispatchFinish {
    context: usize,
}

impl DispatchFinish {
    /// The guard of a dispatch drawing into `context`.
    pub fn new(context: usize) -> Self {
        Self { context }
    }
}

impl Drop for DispatchFinish {
    fn drop(&mut self) {
        // A panic here would abort when the guard runs on unwind, so each step is caught: one failed step does not
        // skip the others.
        let context = self.context;
        let _ = catch_unwind(AssertUnwindSafe(|| UiDrawAdapter::with((), |adapter| adapter.finish(context))));
        let _ = catch_unwind(AssertUnwindSafe(|| TextAdapter::with((), |adapter| adapter.finish(context))));
        let _ = catch_unwind(AssertUnwindSafe(|| ItemDrawAdapter::with((), ItemDrawAdapter::finish)));
    }
}

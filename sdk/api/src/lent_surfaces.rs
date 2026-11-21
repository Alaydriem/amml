use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlDrawSurface, FramePresent, OverlayCopyBackBufferFn};

thread_local! {
    // The surfaces of the `FramePresent` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentSurface>> = const { RefCell::new(Vec::new()) };
}

/// One `FramePresent` call in progress: the address of the event the listener got, the surface as
/// the loader wrote it before the call, and the loader's copy function.
#[derive(Clone, Copy)]
pub(crate) struct LentSurface {
    pub(crate) event: usize,
    pub(crate) surface: AmmlDrawSurface,
    pub(crate) copy_back_buffer: Option<OverlayCopyBackBufferFn>,
}

/// The surfaces the listener trampoline lends for one call. `FramePresentExt::surface` builds a
/// `DrawSurface` only for the event of a running call, and from the copy taken before the call, so
/// a listener that writes the event's surface fields, copies the event or keeps it past the call
/// gets no surface from it.
pub(crate) struct LentSurfaces {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentSurfaces {
    /// For an `E` that is `FramePresent`, records `event` for the call until the returned guard drops;
    /// for any other event it records nothing.
    ///
    /// # Safety
    /// `event` points to a live `E` for the call, of at least `size_of::<E>()` bytes.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn lend<E: 'static>(event: *const E, copy_back_buffer: Option<OverlayCopyBackBufferFn>) -> Self {
        if TypeId::of::<E>() != TypeId::of::<FramePresent>() {
            return Self { pushed: false };
        }
        let present = event.cast::<FramePresent>();
        // SAFETY: `E` is `FramePresent` and the caller guarantees the whole struct is readable.
        let surface = unsafe { core::ptr::addr_of!((*present).surface).read() };
        let entry = LentSurface { event: present.addr(), surface, copy_back_buffer };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// The surface lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find(event: *const FramePresent) -> Option<LentSurface> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).copied()).ok().flatten()
    }
}

impl Drop for LentSurfaces {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

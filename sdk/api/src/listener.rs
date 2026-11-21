use std::ffi::c_void;

use amml_abi::{AmmlStatus, UnlistenFn};

use crate::listener_cell::ListenerCell;

/// One registered listener. Dropping it removes the listener and frees its closure.
///
/// Keep it for as long as the listener should run; the loader also removes every listener of a mod
/// before its `disable` runs.
///
/// A drop waits for calls of the listener in progress on other threads. Do not drop it while
/// holding a lock that the listener takes, and do not let two listeners on two threads drop each
/// other: both wait forever.
pub struct Listener {
    unlisten: UnlistenFn,
    handle: u64,
    user: *mut c_void,
    free: unsafe fn(*mut c_void),
}

// SAFETY: the closure behind `user` is `Send`, and `Listener` only hands the pointer back to the
// loader and frees it once.
#[allow(unsafe_code)]
unsafe impl Send for Listener {}

impl Listener {
    pub(crate) fn new(unlisten: UnlistenFn, handle: u64, user: *mut c_void, free: unsafe fn(*mut c_void)) -> Self {
        Self { unlisten, handle, user, free }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // SAFETY: the loader accepts any handle value and reports an unknown one as a status.
        #[allow(unsafe_code)]
        let status = unsafe { (self.unlisten)(self.handle) };
        // `OK` and `INVALID_ARGUMENT` (removed earlier, for example after `disable`) mean the
        // listener is gone and no call of it runs. On any other status it may still be registered,
        // so the closure is leaked rather than freed under a live callback. The loader never gives
        // handle 0, so a 0 means the handle was not written and proves nothing.
        let gone = self.handle != 0 && (status == AmmlStatus::OK || status == AmmlStatus::INVALID_ARGUMENT);
        // A listener dropped from inside its own closure: the loader does not wait for the caller's
        // own call, so the closure is still running and is leaked.
        if !gone || ListenerCell::<()>::runs_here(self.user) {
            return;
        }
        // SAFETY: `unlisten` returned, and it waits for calls in progress on other threads (a loader
        // guarantee), so no call of this listener runs or can start; the closure does not run on
        // this thread either. `user` came from `ListenerCell::boxed` with the type `free` expects.
        #[allow(unsafe_code)]
        unsafe {
            (self.free)(self.user)
        };
    }
}

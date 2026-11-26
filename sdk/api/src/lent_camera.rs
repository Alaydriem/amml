use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlCameraTable, CameraSetup};

use crate::lent_camera_event::LentCameraEvent;

thread_local! {
    // The `CameraSetup` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentCameraEvent>> = const { RefCell::new(Vec::new()) };
}

/// The camera table the listener trampoline lends for one `CameraSetup` call. `CameraSetupExt::camera` builds a
/// `Camera` only for the event of a running call, so a copy of the event, or one kept past its call, gets none.
pub(crate) struct LentCamera {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentCamera {
    /// For an `E` that is `CameraSetup` and a loader with a camera table, records `event` for the call until the
    /// returned guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(event: *const E, table: Option<AmmlCameraTable>, module: &'static str) -> Self {
        let Some(table) = table.filter(|_| TypeId::of::<E>() == TypeId::of::<CameraSetup>()) else {
            return Self { pushed: false };
        };
        let entry = LentCameraEvent { event: event.addr(), table, module };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// What is lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find(event: *const CameraSetup) -> Option<LentCameraEvent> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).copied()).ok().flatten()
    }
}

impl Drop for LentCamera {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

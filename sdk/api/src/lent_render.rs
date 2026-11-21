use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlRenderTable, WorldRendered};

use crate::lent_render_event::LentRenderEvent;

thread_local! {
    // The `WorldRendered` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentRenderEvent>> = const { RefCell::new(Vec::new()) };
}

/// The render table the listener trampoline lends for one `WorldRendered` call. `WorldRenderedExt::canvas` builds a
/// `WorldCanvas` only for the event of a running call, so a copy of the event, or one kept past its call, gets none.
pub(crate) struct LentRender {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentRender {
    /// For an `E` that is `WorldRendered` and a loader with a render table, records `event` for the call until the
    /// returned guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(event: *const E, table: Option<AmmlRenderTable>, module: &'static str) -> Self {
        let Some(table) = table.filter(|_| TypeId::of::<E>() == TypeId::of::<WorldRendered>()) else {
            return Self { pushed: false };
        };
        let entry = LentRenderEvent { event: event.addr(), table, module };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// What is lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find(event: *const WorldRendered) -> Option<LentRenderEvent> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).copied()).ok().flatten()
    }
}

impl Drop for LentRender {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

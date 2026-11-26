use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlRenderTable, BlockEntityPass, WorldRendered};

use crate::lent_render_event::LentRenderEvent;

thread_local! {
    // The `WorldRendered` and `BlockEntityPass` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentRenderEvent>> = const { RefCell::new(Vec::new()) };
}

/// The render table the listener trampoline lends for one `WorldRendered` or `BlockEntityPass` call.
/// `WorldRenderedExt::canvas` builds a `WorldCanvas`, and `BlockEntityPassExt::draw_block_entity` draws, only for the event
/// of a running call, so a copy of the event, or one kept past its call, gets none.
pub(crate) struct LentRender {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentRender {
    /// For an `E` that is `WorldRendered` or `BlockEntityPass` and a loader with a render table, records `event` for the
    /// call until the returned guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(event: *const E, table: Option<AmmlRenderTable>, module: &'static str) -> Self {
        let lent =
            TypeId::of::<E>() == TypeId::of::<WorldRendered>() || TypeId::of::<E>() == TypeId::of::<BlockEntityPass>();
        let Some(table) = table.filter(|_| lent) else {
            return Self { pushed: false };
        };
        let entry = LentRenderEvent { event: event.addr(), table, module };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// What is lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find<E>(event: *const E) -> Option<LentRenderEvent> {
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

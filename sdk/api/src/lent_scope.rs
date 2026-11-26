use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlInventoryTable, AmmlUiTable, ScreenRender};

use crate::lent_view::LentView;

thread_local! {
    // The `ScreenRender` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentView>> = const { RefCell::new(Vec::new()) };
}

/// The ui table the listener trampoline lends for one `ScreenRender` call. `ScreenRenderExt::view` builds a
/// `ScreenView` only for the event of a running call, so a copy of the event, or one kept past its call, gets none.
pub(crate) struct LentScope {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentScope {
    /// For an `E` that is `ScreenRender` and a loader with a ui table, records `event` for the call until the
    /// returned guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(
        event: *const E,
        table: Option<AmmlUiTable>,
        inventory: Option<AmmlInventoryTable>,
        module: &'static str,
    ) -> Self {
        let Some(table) = table.filter(|_| TypeId::of::<E>() == TypeId::of::<ScreenRender>()) else {
            return Self { pushed: false };
        };
        let entry = LentView { event: event.addr(), table, inventory, module };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// The ui table lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find(event: *const ScreenRender) -> Option<AmmlUiTable> {
        Self::find_view(event).map(|view| view.table)
    }

    /// Everything lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find_view(event: *const ScreenRender) -> Option<LentView> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).copied()).ok().flatten()
    }
}

impl Drop for LentScope {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

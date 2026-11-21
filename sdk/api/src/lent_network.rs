use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlNetworkTable, PlayerListUpdate, PlayerSkinChanged};

use crate::lent_network_event::LentNetworkEvent;

thread_local! {
    // The network events running on this thread, innermost last.
    static LENT: RefCell<Vec<LentNetworkEvent>> = const { RefCell::new(Vec::new()) };
}

/// The network table the listener trampoline lends for one `PlayerListUpdate` or `PlayerSkinChanged` call. The event
/// reads build entries and skins only for the event of a running call, so a copy of the event, or one kept past its
/// call, gets none.
pub(crate) struct LentNetwork {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentNetwork {
    /// For an `E` that is a network event and a loader with a network table, records `event` for the call until the
    /// returned guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(event: *const E, table: Option<AmmlNetworkTable>) -> Self {
        let is_network = TypeId::of::<E>() == TypeId::of::<PlayerListUpdate>()
            || TypeId::of::<E>() == TypeId::of::<PlayerSkinChanged>();
        let Some(table) = table.filter(|_| is_network) else { return Self { pushed: false } };
        let entry = LentNetworkEvent { event: event.addr(), table };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// The network table lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find<E>(event: *const E) -> Option<AmmlNetworkTable> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).map(|entry| entry.table))
            .ok()
            .flatten()
    }
}

impl Drop for LentNetwork {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

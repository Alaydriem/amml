use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlChatField, AmmlStatus, AmmlStr, AmmlUiTable, ChatMessageAdded};

use crate::ApiError;
use crate::lent_chat_event::LentChatEvent;

thread_local! {
    // The `ChatMessageAdded` calls running on this thread, innermost last.
    static LENT: RefCell<Vec<LentChatEvent>> = const { RefCell::new(Vec::new()) };
}

/// The ui table the listener trampoline lends for one `ChatMessageAdded` call. The reads and edits of the event answer only
/// for the event of a running call, so a copy of the event, or one kept past its call, gets none.
pub(crate) struct LentChat {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentChat {
    /// For an `E` that is `ChatMessageAdded` and a loader with a ui table, records `event` for the call until the returned
    /// guard drops; for any other event it records nothing.
    pub(crate) fn lend<E: 'static>(event: *const E, table: Option<AmmlUiTable>, module: &'static str) -> Self {
        let Some(table) = table.filter(|_| TypeId::of::<E>() == TypeId::of::<ChatMessageAdded>()) else {
            return Self { pushed: false };
        };
        let entry = LentChatEvent { event: event.addr(), table, module };
        let pushed = LENT.try_with(|lent| lent.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// What is lent for the event at `event`, when its call runs on this thread.
    pub(crate) fn find(event: *const ChatMessageAdded) -> Option<LentChatEvent> {
        let address = event.addr();
        LENT.try_with(|lent| lent.borrow().iter().rev().find(|entry| entry.event == address).copied()).ok().flatten()
    }

    /// One string of the running event `event`; the bytes are the game's, valid until the last listener returns, which
    /// the borrow of the event covers.
    #[allow(unsafe_code)]
    pub(crate) fn read(event: &ChatMessageAdded, field: AmmlChatField) -> Option<&str> {
        let lent = Self::find(std::ptr::from_ref(event))?;
        let chat_text = lent.table.chat_text?;
        let mut out = AmmlStr::EMPTY;
        // SAFETY: the table function is the loader's; `out` is writable.
        let status = unsafe { chat_text(field, &mut out) };
        if status != AmmlStatus::OK {
            return None;
        }
        // SAFETY: the view of the loader is valid until the event ends; text that is not UTF-8 reads as `None`.
        unsafe { out.as_str() }.ok()
    }

    /// Writes `text` into `field` of the running event `event`; `Refused` outside its call.
    #[allow(unsafe_code)]
    pub(crate) fn write(event: &ChatMessageAdded, field: AmmlChatField, text: &str) -> Result<(), ApiError> {
        let lent = Self::find(std::ptr::from_ref(event)).ok_or(ApiError::Refused)?;
        let set = lent.table.chat_set_text.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the module id and the text live for the call.
        let status = unsafe { set(AmmlStr::from_str(lent.module), field, AmmlStr::from_str(text)) };
        ApiError::check(status)
    }
}

impl Drop for LentChat {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|lent| lent.borrow_mut().pop());
        }
    }
}

use std::any::TypeId;
use std::cell::RefCell;

use amml_abi::{AmmlStr, JoinStarted, ServerJoinStarting, WorldStorageOpened};

// The most string views one lending event carries (`ServerJoinStarting`: host, url, name).
const MAX_VIEWS: usize = 3;

type Views = [(usize, usize); MAX_VIEWS];

thread_local! {
    // The events running on this thread that lend text, innermost last: the address of the event the listener got and
    // the address and length of each string view the loader lent with it.
    static LENT: RefCell<Vec<(usize, Views)>> = const { RefCell::new(Vec::new()) };
}

/// The text views the listener trampoline lends for one `JoinStarted`, `ServerJoinStarting` or `WorldStorageOpened`
/// call. The `Ext` reads give the text only for the event of a running call, from the views taken before the call, so a
/// copy of the event, one kept past its call, or a listener that wrote the fields gets none.
pub(crate) struct LentStrings {
    // Whether `lend` pushed an entry that the drop pops.
    pushed: bool,
}

impl LentStrings {
    /// For an `E` that is `JoinStarted`, `ServerJoinStarting` or `WorldStorageOpened`, records `event` and its string
    /// views for the call until the returned guard drops; for any other event it records nothing.
    ///
    /// # Safety
    /// `event` points to a live `E` for the call, of at least `size_of::<E>()` bytes.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn lend<E: 'static>(event: *const E) -> Self {
        let views: [AmmlStr; MAX_VIEWS] = if TypeId::of::<E>() == TypeId::of::<JoinStarted>() {
            let join = event.cast::<JoinStarted>();
            // SAFETY: `E` is `JoinStarted` and the caller guarantees the whole struct is readable.
            unsafe {
                [
                    core::ptr::addr_of!((*join).server_name).read(),
                    core::ptr::addr_of!((*join).world_name).read(),
                    AmmlStr::EMPTY,
                ]
            }
        } else if TypeId::of::<E>() == TypeId::of::<ServerJoinStarting>() {
            let join = event.cast::<ServerJoinStarting>();
            // SAFETY: `E` is `ServerJoinStarting` and the caller guarantees the whole struct is readable.
            unsafe {
                [
                    core::ptr::addr_of!((*join).host).read(),
                    core::ptr::addr_of!((*join).url).read(),
                    core::ptr::addr_of!((*join).name).read(),
                ]
            }
        } else if TypeId::of::<E>() == TypeId::of::<WorldStorageOpened>() {
            let opened = event.cast::<WorldStorageOpened>();
            // SAFETY: `E` is `WorldStorageOpened` and the caller guarantees the whole struct is readable.
            unsafe { [core::ptr::addr_of!((*opened).path).read(), AmmlStr::EMPTY, AmmlStr::EMPTY] }
        } else {
            return Self { pushed: false };
        };
        let entry = (event.addr(), views.map(|view| (view.ptr.expose_provenance(), view.len)));
        let pushed = LENT.try_with(|cell| cell.borrow_mut().push(entry)).is_ok();
        Self { pushed }
    }

    /// The text lent as view `index` with the event at `event`, when its call runs on this thread; `None` for no call,
    /// a null or empty view, or text that is not UTF-8.
    pub(crate) fn find<E>(event: &E, index: usize) -> Option<&str> {
        let address = std::ptr::from_ref(event).addr();
        let views = LENT
            .try_with(|cell| cell.borrow().iter().rev().find(|entry| entry.0 == address).map(|entry| entry.1))
            .ok()
            .flatten()?;
        let (at, len) = *views.get(index)?;
        if at == 0 || len == 0 {
            return None;
        }
        // SAFETY: the entry was recorded for the call of the event at `event`, which runs on this thread now: the loader
        // lends `len` bytes at `at` until that call returns, and the borrow of `event` ends inside it.
        #[allow(unsafe_code)]
        let bytes = unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(at), len) };
        std::str::from_utf8(bytes).ok()
    }
}

impl Drop for LentStrings {
    fn drop(&mut self) {
        if self.pushed {
            let _ = LENT.try_with(|cell| cell.borrow_mut().pop());
        }
    }
}

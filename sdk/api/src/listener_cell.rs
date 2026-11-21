use std::cell::RefCell;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

use amml_abi::{AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlThread};

use crate::client_tables::ClientTables;
use crate::lent_chat::LentChat;
use crate::lent_network::LentNetwork;
use crate::lent_scope::LentScope;
use crate::lent_surfaces::LentSurfaces;
use crate::lent_tables::LentTables;
use crate::{Event, EventCtx, Logger, PanicText, WorldEpoch};

thread_local! {
    // The cells whose closure runs on this thread, innermost last.
    static RUNNING: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// The boxed closure of one listener, behind the `user` pointer the loader passes back to the
/// trampoline, with the client table and mod id the trampoline puts in the `EventCtx`, and the loader functions
/// that a `FramePresent` surface and a `ScreenRender` view lend.
pub(crate) struct ListenerCell<F> {
    call: Mutex<F>,
    log: Logger,
    event: &'static str,
    client: Option<ClientTables>,
    lent: LentTables,
    module: &'static str,
}

impl<F: Send + 'static> ListenerCell<F> {
    /// Boxes `call` and gives the pointer to pass to the loader as `user`. `free` releases it.
    pub(crate) fn boxed(
        call: F,
        log: Logger,
        event: &'static str,
        client: Option<ClientTables>,
        lent: LentTables,
        module: &'static str,
    ) -> *mut c_void {
        let cell = Self { call: Mutex::new(call), log, event, client, lent, module };
        Box::into_raw(Box::new(cell)).cast::<c_void>()
    }

    /// Releases a pointer from `boxed`.
    ///
    /// # Safety
    /// `user` comes from `boxed` with the same `F`, is freed once, and no call of the trampoline
    /// with it runs or can start.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn free(user: *mut c_void) {
        // SAFETY: the caller guarantees `user` is a live box of `Self` that nothing else uses.
        drop(unsafe { Box::from_raw(user.cast::<Self>()) });
    }

    /// The `EventCallbackFn` for event `E`. Runs the closure inside `catch_unwind`: a panic is
    /// logged and gives `PANICKED`, and never unwinds into the loader. A call that re-enters the
    /// same listener on one thread returns `OK` without running the closure again. For a
    /// `FramePresent` the surface is lent for the call (`LentSurfaces`), for a `ScreenRender` the view (`LentScope`).
    ///
    /// # Safety
    /// The loader calls it with the `user` pointer of a live `boxed` cell for this `F`, an event
    /// struct of at least its header's `size` bytes, and a context that lives for the call.
    #[allow(unsafe_code)]
    pub(crate) unsafe extern "C" fn trampoline<E: Event>(
        event: *mut AmmlEventHeader,
        context: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus
    where
        F: FnMut(&mut E, &EventCtx),
    {
        if event.is_null() || user.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: `event` is non-null and starts with an `AmmlEventHeader`.
        let size = unsafe { core::ptr::addr_of!((*event).size).read() };
        if (size as usize) < std::mem::size_of::<E>() {
            return AmmlStatus::VERSION_MISMATCH;
        }
        let key = user as usize;
        if !Self::enter(key) {
            return AmmlStatus::OK;
        }
        // SAFETY: `user` is a live cell of this `F` for the whole call (the loader guarantee).
        let cell = unsafe { &*user.cast::<Self>() };
        // SAFETY: the loader passes the context it owns for the call, or null.
        let context = unsafe { Self::context(context, cell.client, cell.module) };
        // SAFETY: the header says the struct holds at least `size_of::<E>()` bytes, live for the call.
        let lent = unsafe { LentSurfaces::lend(event.cast::<E>().cast_const(), cell.lent.copy_back_buffer) };
        let view = LentScope::lend(event.cast::<E>().cast_const(), cell.lent.ui, cell.lent.inventory, cell.module);
        let network = LentNetwork::lend(event.cast::<E>().cast_const(), cell.lent.network);
        let chat = LentChat::lend(event.cast::<E>().cast_const(), cell.lent.ui, cell.module);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let mut call = cell.call.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            // SAFETY: the header says the struct holds at least `size_of::<E>()` bytes, and by the
            // `Event` contract it is laid out as `E`; the loader gives this call sole access.
            let event = unsafe { &mut *event.cast::<E>() };
            (call)(event, &context);
        }));
        drop(chat);
        drop(network);
        drop(view);
        drop(lent);
        Self::leave(key);
        match outcome {
            Ok(()) => AmmlStatus::OK,
            Err(payload) => {
                PanicText::consume(payload, |text| {
                    cell.log.error("a listener panicked", &[("event", cell.event), ("panic", text)]);
                });
                AmmlStatus::PANICKED
            }
        }
    }

    /// Reads the loader's context. A null context, or one shorter than its v1 size, gives the game
    /// thread, no game, epoch 0 and no client; a context longer than this crate knows is read up to its
    /// v1 fields. The client handle is paired with the loader's client table, when there is one.
    ///
    /// # Safety
    /// A non-null `context` points to at least its `size` readable bytes.
    #[allow(unsafe_code)]
    unsafe fn context(
        context: *const AmmlEventContext,
        client: Option<ClientTables>,
        module: &'static str,
    ) -> EventCtx {
        let fallback = EventCtx::new(AmmlThread::GAME, false, WorldEpoch(0));
        if context.is_null() {
            return fallback;
        }
        // SAFETY: `context` is non-null and its leading `size` field is readable.
        let size = unsafe { core::ptr::addr_of!((*context).size).read() };
        if size < AmmlEventContext::SIZE_V1 {
            return fallback;
        }
        // SAFETY: `size` covers every v1 field, which are the fields read here.
        unsafe {
            EventCtx::new(
                core::ptr::addr_of!((*context).thread).read(),
                core::ptr::addr_of!((*context).game_alive).read() != 0,
                WorldEpoch(core::ptr::addr_of!((*context).epoch).read()),
            )
            .with_client(core::ptr::addr_of!((*context).client).read(), client, module)
        }
    }

    // Marks `key` running on this thread; false when it already runs here.
    fn enter(key: usize) -> bool {
        RUNNING
            .try_with(|running| {
                let mut running = running.borrow_mut();
                if running.contains(&key) {
                    return false;
                }
                running.push(key);
                true
            })
            .unwrap_or(false)
    }

    fn leave(key: usize) {
        let _ = RUNNING.try_with(|running| {
            let mut running = running.borrow_mut();
            if let Some(at) = running.iter().rposition(|held| *held == key) {
                running.remove(at);
            }
        });
    }
}

impl ListenerCell<()> {
    /// Whether the closure behind `user` runs on this thread, so freeing it now would free a running
    /// closure. Assumes it runs when the thread state cannot be read.
    pub(crate) fn runs_here(user: *mut c_void) -> bool {
        RUNNING.try_with(|running| running.borrow().contains(&(user as usize))).unwrap_or(true)
    }
}

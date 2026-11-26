use std::any::TypeId;

use amml_abi::{AmmlStr, FramePresent, ListenFn, UnlistenFn};

use crate::client_tables::ClientTables;
use crate::lent_tables::LentTables;
use crate::listener_cell::ListenerCell;
use crate::{ApiError, Event, EventCtx, HookOrder, Listener, Logger, Uses};

/// Registers listeners for loader and game events.
#[derive(Clone, Copy)]
pub struct Events {
    listen: ListenFn,
    unlisten: UnlistenFn,
    uses: Uses,
    log: Logger,
    client: Option<ClientTables>,
    lent: LentTables,
    module: &'static str,
}

impl Events {
    pub(crate) fn new(
        listen: ListenFn,
        unlisten: UnlistenFn,
        uses: Uses,
        log: Logger,
        client: Option<ClientTables>,
        lent: LentTables,
        module: &'static str,
    ) -> Self {
        Self { listen, unlisten, uses, log, client, lent, module }
    }

    /// Calls `f` for every `E` until the returned `Listener` is dropped. The listen is required when
    /// a feature of the mod's `amml_uses!` names `E` as required, otherwise optional. An event the
    /// loader cannot serve gives `NotAvailable`.
    ///
    /// A panic in `f` is caught and logged; the loader then disables the mod.
    pub fn listen<E: Event>(
        &self,
        order: HookOrder,
        f: impl FnMut(&mut E, &EventCtx) + Send + 'static,
    ) -> Result<Listener, ApiError> {
        self.register(order, f)
    }

    fn register<E: Event, F: FnMut(&mut E, &EventCtx) + Send + 'static>(
        &self,
        order: HookOrder,
        f: F,
    ) -> Result<Listener, ApiError> {
        let user = ListenerCell::boxed(f, self.log, E::ID, self.client, self.lent, self.module);
        let required = u8::from(self.uses.requires(E::ID));
        let mut handle = 0u64;
        // SAFETY: the id lives for the call, `trampoline` matches `EventCallbackFn` and expects the
        // `user` box of this `F`, and `handle` is a live u64.
        #[allow(unsafe_code)]
        let status = unsafe {
            (self.listen)(
                AmmlStr::from_str(E::ID),
                self.version_of::<E>(),
                order.0,
                required,
                Some(ListenerCell::<F>::trampoline::<E>),
                user,
                &mut handle,
            )
        };
        if !status.is_ok() {
            // SAFETY: the loader refused the listener, so it never calls the trampoline with `user`.
            #[allow(unsafe_code)]
            unsafe {
                ListenerCell::<F>::free(user)
            };
            return Err(ApiError::from(status));
        }
        Ok(Listener::new(self.unlisten, handle, user, ListenerCell::<F>::free))
    }

    // The version to listen to `E` at: `FramePresent::ON_DEMAND_VERSION` for a `FramePresent` when the loader serves
    // it (`DrawSurface` then asks for the surface on first use), else the version of `E`.
    fn version_of<E: Event>(&self) -> u32 {
        if TypeId::of::<E>() == TypeId::of::<FramePresent>() && self.lent.prepare_surface.is_some() {
            FramePresent::ON_DEMAND_VERSION
        } else {
            E::VERSION
        }
    }
}

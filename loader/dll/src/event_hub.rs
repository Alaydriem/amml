use std::ffi::c_void;
use std::sync::Mutex;

use amml_abi::{
    AmmlBand, AmmlEvent, AmmlEventContext, AmmlEventHeader, AmmlOrigin, AmmlPhase, AmmlStatus, AmmlThread,
    BlockChanged, CameraInstructionReceived, CameraSetup, ChatMessageAdded, ContainerSlotHover, ContainerSlotUnhover,
    FrameUpdate, ItemHovertext, LocatorBarIconDraw, PlayerListUpdate, PlayerSkinChanged, ScreenChanged, ScreenRender,
    UiPointer, UiScroll, WorldRendered,
};
use amml_data::{AdapterCatalog, NeedKind};
use amml_runtime::{CallOutcome, DispatchReport, EventBus, HeaderStamp, ListenError, ListenerId};

use crate::{
    BlockListener, Boundary, CameraDetours, ChatDetours, CurrentMod, DeferredDisable, GameGate, HandlerDetours,
    HovertextDetours, LocatorBarDetours, ModListener, PacketDetours, RenderLevelDetours, ScreenDetours, UiInputDetours,
};

// The pause of a split raise: the band it follows and what runs there.
type Between<'a, E> = (AmmlBand, &'a mut dyn FnMut(&mut E));

/// The loader's event bus with the mod listeners, the game gate and the mods waiting to be disabled
/// after a listener panic.
pub struct EventHub {
    bus: EventBus<ModListener>,
    deferred: DeferredDisable,
    gate: GameGate,
    // Serializes the refresh of the screen detour flag, so the last refresh sees every change before it.
    refresh: Mutex<()>,
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHub {
    /// The injector of an injected event with no single injector. No mod index equals it, so every listener
    /// sees `OTHER_MOD`.
    pub const NO_INJECTOR: usize = usize::MAX;
    const CONTEXT_VERSION: u32 = 1;
    const LOADER_SHAPE: &'static str = "loader";

    /// A hub that offers only the events the loader raises itself (shape `loader` in the catalog).
    /// Game events are offered by their adapters.
    pub fn new() -> Self {
        let hub = Self {
            bus: EventBus::new(),
            deferred: DeferredDisable::new(),
            gate: GameGate::new(),
            refresh: Mutex::new(()),
        };
        for need in AdapterCatalog::needs() {
            if need.kind == NeedKind::Event && need.shapes.iter().any(|shape| shape.name == Self::LOADER_SHAPE) {
                hub.offer(need.id, need.version);
            }
        }
        hub
    }

    /// Marks `event` available up to `version`.
    pub fn offer(&self, event: &str, version: u32) {
        self.bus.offer(event, version);
    }

    pub fn is_available(&self, event: &str, version: u32) -> bool {
        self.bus.is_available(event, version)
    }

    pub fn listen(
        &self,
        event: &str,
        version: u32,
        band: AmmlBand,
        listener: ModListener,
    ) -> Result<ListenerId, ListenError> {
        let added = self.bus.add(event, version, band.0, listener.owner, listener);
        self.refresh_flags();
        added
    }

    /// Removes one listener and waits for its calls on other threads. False for an unknown id.
    pub fn unlisten(&self, id: ListenerId) -> bool {
        let removed = self.bus.remove(id);
        self.refresh_flags();
        removed
    }

    /// Removes every listener of the mod at `owner`, waits for their calls and refuses new listens
    /// of it until `reopen_owner`.
    pub fn remove_owner(&self, owner: usize) -> usize {
        let removed = self.bus.remove_owner(owner);
        self.refresh_flags();
        removed
    }

    pub fn reopen_owner(&self, owner: usize) {
        self.bus.reopen_owner(owner);
    }

    pub fn deferred(&self) -> &DeferredDisable {
        &self.deferred
    }

    pub fn gate(&self) -> &GameGate {
        &self.gate
    }

    /// A header for an event of type `E` raised by the game or the loader.
    pub fn header<E: AmmlEvent>(phase: AmmlPhase) -> AmmlEventHeader {
        AmmlEventHeader {
            size: std::mem::size_of::<E>() as u32,
            version: E::VERSION,
            phase,
            origin: AmmlOrigin::GAME,
            cancelled: 0,
            injected: 0,
            _reserved: 0,
        }
    }

    /// The context of an event the loader raises on the game thread: no client, the current epoch
    /// and `game_alive` from the gate.
    pub fn loader_context(&self) -> AmmlEventContext {
        self.context_on(AmmlThread::GAME)
    }

    /// The context of an event the loader raises on `thread`: no client, the current epoch and
    /// `game_alive` from the gate.
    pub fn context_on(&self, thread: AmmlThread) -> AmmlEventContext {
        AmmlEventContext {
            size: std::mem::size_of::<AmmlEventContext>() as u32,
            version: Self::CONTEXT_VERSION,
            thread,
            game_alive: u8::from(self.gate.is_open()),
            _reserved: [0; 6],
            epoch: self.gate.epoch().0,
            client: std::ptr::null(),
        }
    }

    /// Calls every listener of `E` with `value` and `context`. A `FrameUpdate` opens the game gate
    /// first. Listeners run with no current mod, so a listen or provide from inside one is refused;
    /// a listener that returns `PANICKED` (or panics in the loader) is reported and its mod queued
    /// in `deferred`.
    ///
    /// The header the loader filled is written back before and after every listener call
    /// (`HeaderStamp`): a listener cannot change it for later listeners or the loader, a cancel of an
    /// `E::CANCELLABLE` event stays, and any other event is never cancelled.
    ///
    /// `E` must start with an `AmmlEventHeader`, as every `amml_abi::events` struct does.
    pub fn raise<E: AmmlEvent + Clone>(&self, value: &mut E, context: &AmmlEventContext) -> DispatchReport {
        self.raise_skipping(value, context, None)
    }

    /// Like `raise`, and the listeners of the mod at `skip` are not called.
    pub fn raise_skipping<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        skip: Option<usize>,
    ) -> DispatchReport {
        self.dispatch(value, context, None, skip, None, true, None, None)
    }

    /// Like `raise`, and `keep` runs after every listener call: the fields it writes are the loader's, so no
    /// listener changes them for a later listener or for the loader.
    pub fn raise_keeping<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        keep: &dyn Fn(&mut E),
    ) -> DispatchReport {
        self.dispatch(value, context, None, None, None, true, Some(keep), None)
    }

    /// Like `raise`, with a pause: `between` runs once after the listeners of band `after_band` and before those of
    /// the next band, also when either side has no listener. The value `between` gets is the one the earlier bands
    /// left.
    pub fn raise_split<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        after_band: AmmlBand,
        mut between: impl FnMut(&mut E),
    ) -> DispatchReport {
        self.dispatch(value, context, None, None, None, true, None, Some((after_band, &mut between)))
    }

    /// `raise_split` and `raise_keeping` together: `keep` runs after every listener call, `between` once after band
    /// `after_band`.
    pub fn raise_split_keeping<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        keep: &dyn Fn(&mut E),
        after_band: AmmlBand,
        mut between: impl FnMut(&mut E),
    ) -> DispatchReport {
        self.dispatch(value, context, None, None, None, true, Some(keep), Some((after_band, &mut between)))
    }

    /// True when `event` has at least one listener now.
    pub fn has_listeners(&self, event: &str) -> bool {
        self.bus.has_listeners(event)
    }

    // Tells the screen detours whether a listener of `ScreenRender`, `ScreenChanged` or a container hover event exists,
    // after every change of the listener set. A detour reads that one flag and nothing else on its hot path.
    fn refresh_flags(&self) {
        let _guard = self.refresh.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let wanted = self.bus.has_listeners(ScreenRender::ID) || self.bus.has_listeners(ScreenChanged::ID);
        ScreenDetours::set_views_wanted(wanted);
        let hover = self.bus.has_listeners(ContainerSlotHover::ID) || self.bus.has_listeners(ContainerSlotUnhover::ID);
        ScreenDetours::set_hover_wanted(hover);
        UiInputDetours::set_pointer_wanted(self.bus.has_listeners(UiPointer::ID));
        UiInputDetours::set_scroll_wanted(self.bus.has_listeners(UiScroll::ID));
        HovertextDetours::set_wanted(self.bus.has_listeners(ItemHovertext::ID));
        PacketDetours::set_list_wanted(self.bus.has_listeners(PlayerListUpdate::ID));
        PacketDetours::set_skin_wanted(self.bus.has_listeners(PlayerSkinChanged::ID));
        LocatorBarDetours::set_wanted(self.bus.has_listeners(LocatorBarIconDraw::ID));
        ChatDetours::set_wanted(self.bus.has_listeners(ChatMessageAdded::ID));
        BlockListener::set_wanted(self.bus.has_listeners(BlockChanged::ID));
        RenderLevelDetours::set(RenderLevelDetours::LISTENER, self.bus.has_listeners(WorldRendered::ID));
        CameraDetours::set(CameraDetours::LISTENER, self.bus.has_listeners(CameraSetup::ID));
        HandlerDetours::set(HandlerDetours::LISTENER, self.bus.has_listeners(CameraInstructionReceived::ID));
    }

    /// Like `raise`, for a message the loader passes on whatever the listeners say: a cancel is
    /// ignored and every listener sees `cancelled` 0.
    pub fn raise_fixed<E: AmmlEvent + Clone>(&self, value: &mut E, context: &AmmlEventContext) -> DispatchReport {
        self.dispatch(value, context, None, None, None, false, None, None)
    }

    /// Like `raise`, and only the listeners of the mod at `owner` are called.
    pub fn raise_only<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        owner: usize,
    ) -> DispatchReport {
        self.dispatch(value, context, Some(owner), None, None, true, None, None)
    }

    /// Raises an event the loader injected for the mod at `injector`: `injected` is 1, `origin` is
    /// `THIS_MOD` for the listeners of `injector` and `OTHER_MOD` for the others, and a cancel is
    /// ignored (the event is not cancellable, so a cancel stops no later listener). An event with no
    /// single injector is raised on `NO_INJECTOR`.
    pub fn raise_injected<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        injector: usize,
    ) -> DispatchReport {
        Self::header_of(value).injected = 1;
        self.dispatch(value, context, None, None, Some(injector), true, None, None)
    }

    /// `raise_injected` and `raise_keeping` together: `keep` runs after every listener call.
    pub fn raise_injected_keeping<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        injector: usize,
        keep: &dyn Fn(&mut E),
    ) -> DispatchReport {
        Self::header_of(value).injected = 1;
        self.dispatch(value, context, None, None, Some(injector), true, Some(keep), None)
    }

    // `only` and `skip` select listeners by owner; `injector` is set for an injected event; `may_cancel`
    // false ignores every cancel; `keep` runs after each listener call; `split` runs its closure once after the
    // listeners of its band.
    #[allow(clippy::too_many_arguments)]
    fn dispatch<E: AmmlEvent + Clone>(
        &self,
        value: &mut E,
        context: &AmmlEventContext,
        only: Option<usize>,
        skip: Option<usize>,
        injector: Option<usize>,
        may_cancel: bool,
        keep: Option<&dyn Fn(&mut E)>,
        split: Option<Between<'_, E>>,
    ) -> DispatchReport {
        const {
            assert!(std::mem::size_of::<E>() >= std::mem::size_of::<AmmlEventHeader>());
            assert!(std::mem::align_of::<E>() >= std::mem::align_of::<AmmlEventHeader>());
        }
        if E::ID == FrameUpdate::ID {
            self.gate.open();
        }
        let cancellable = E::CANCELLABLE && injector.is_none() && may_cancel;
        let mut stamp = HeaderStamp::new(*Self::header_of(value), cancellable);
        let call = |listener: &ModListener, value: &mut E| {
            if Some(listener.owner) == skip || only.is_some_and(|only| only != listener.owner) {
                CallOutcome::Done
            } else {
                let origin = match injector {
                    None => None,
                    Some(injector) if injector == listener.owner => Some(AmmlOrigin::THIS_MOD),
                    Some(_) => Some(AmmlOrigin::OTHER_MOD),
                };
                let outcome = Self::call(listener, value, context, &mut stamp, origin);
                if let Some(keep) = keep {
                    keep(value);
                }
                outcome
            }
        };
        let report = CurrentMod::outside(|| match split {
            None => self.bus.dispatch(E::ID, value, call),
            Some((band, between)) => self.bus.dispatch_split(E::ID, value, band.0, call, between),
        });
        stamp.stamp(Self::header_of(value));
        for owner in &report.panicked_owners {
            self.deferred.push(*owner);
        }
        report
    }

    // The header at the start of `value`.
    fn header_of<E>(value: &mut E) -> &mut AmmlEventHeader {
        // SAFETY: `E` starts with an `AmmlEventHeader` and is at least as large and as aligned
        // (the `const` asserts in `dispatch`); the borrow of `value` covers the header.
        unsafe { &mut *std::ptr::from_mut(value).cast::<AmmlEventHeader>() }
    }

    fn call<E>(
        listener: &ModListener,
        value: &mut E,
        context: &AmmlEventContext,
        stamp: &mut HeaderStamp,
        origin: Option<AmmlOrigin>,
    ) -> CallOutcome {
        match origin {
            Some(origin) => *Self::header_of(value) = stamp.with_origin(origin),
            None => stamp.stamp(Self::header_of(value)),
        }
        let event = std::ptr::from_mut(value).cast::<AmmlEventHeader>();
        let user = std::ptr::with_exposed_provenance_mut::<c_void>(listener.user);
        // SAFETY: `event` points to a live `E` that starts with an `AmmlEventHeader` (the `const` asserts
        // in `dispatch`), `context` lives for the call, and `user` is the pointer the mod registered with
        // this callback.
        let status = Boundary::guard(|| unsafe { (listener.callback)(event, context, user) });
        if status == AmmlStatus::PANICKED {
            return CallOutcome::Panicked;
        }
        if stamp.after_call(Self::header_of(value)) { CallOutcome::Cancelled } else { CallOutcome::Done }
    }
}

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::path::PathBuf;
use std::rc::Rc;

use amml_abi::{
    AmmlApi, AmmlDirKind, AmmlEventContext, AmmlEventHeader, AmmlMouseAction, AmmlMouseButton, AmmlMoveIntent,
    AmmlOrigin, AmmlPhase, AmmlServices, AmmlStatus, AmmlThread, AmmlUseRow, InjectedInputReleased, InputLeaseLost,
    InputLeaseResumed, InputLeaseSuspended, InventoryRequestDone, KeyInput, LookDelta, MouseInput,
};
use amml_runtime::{CallOutcome, HeaderStamp, InjectedOp, NoticeKind};

use crate::testing::{
    FakeClient, FakeDispatch, FakeListener, FakePhase, FakeScreen, FakeState, FakeTables, FakeWindow,
};
use crate::{AmmlMod, ApiError, Area, Ctx, Event, ShotLayers, Uses};

/// A loader for mod tests. `ctx` gives a `Ctx` over a real v3 table whose functions call into this
/// fake, so the code under test runs the same `Ctx` paths as in the game.
///
/// The table functions find the fake through a thread-local: create, use and drop a `FakeGame` on
/// one thread. A thread holds one live `FakeGame` at a time; `new` panics on a second. A listener
/// handle, a lease id and a hold id carry the tag of their fake, so one dropped where another fake is
/// current changes nothing in that fake.
///
/// A test of a whole mod drives it with `load`, `enable` and `disable`, which apply the loader's
/// rules: `listen` and `provide` work only inside `load` and `enable`. Until the first driver call
/// the fake accepts them at any time, for tests of one piece of a mod.
///
/// Areas: like the loader, the fake offers only `input.lease` until a test offers more with
/// `offer_area`. Client calls check the gate, the handle and the area of the call; injection checks
/// the gate, the area, the lease and whether `bind_input` ran, in the loader's order. Window calls
/// check the area and whether `set_window` ran; screenshots and capture streams check the area, the
/// request, then the gate (which stands for the bound swap chain).
pub struct FakeGame {
    state: Rc<FakeState>,
    tables: FakeTables,
}

impl Default for FakeGame {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeGame {
    /// The most wheel notches one queued wheel sends, as in the loader.
    const MAX_WHEEL_NOTCHES: u32 = 100;

    /// A fake with no events offered, the gate closed, epoch 0 and v8 services (client, input, window,
    /// overlay, actor, inventory, lighting, ui and network tables).
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn new() -> Self {
        Self::with_services(AmmlServices::SIZE_V8)
    }

    /// A fake whose services table is v7: every area table but the network table, as a loader before it. Its
    /// network events carry no entries and no skin, and `Ctx::supports` is false for every `network.*` area.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v7_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V7)
    }

    /// A fake whose services table is v6: every area table but the ui table, as a loader before it. Its
    /// `ScreenView` reads give `None` and `Ctx::supports(Area::UI_SCREEN)` is false.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v6_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V6)
    }

    /// A fake whose services table is v5: every area table but the lighting table, as a loader before it. Its
    /// `Ctx::lighting` calls give `NotAvailable`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v5_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V5)
    }

    /// A fake whose services table is v4: every area table but the inventory table, as a loader before it. Its
    /// `LocalPlayer::held_item` and `inventory` give `None`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v4_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V4)
    }

    /// A fake whose services table is v3: every area table but the actor table, as a loader before it. Its
    /// `LocalPlayer` reads give `None`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v3_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V3)
    }

    /// A fake whose services table is v2: client and input tables, no window and no overlay table, as a
    /// loader before them. Its `Window` and `Overlay` give `None`, 0 or `NotAvailable`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v2_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V2)
    }

    /// A fake whose services table is v1: no client and no input table, as a loader before them. Its
    /// `Ctx` supports no client or input area and its events carry no `Client`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v1_services() -> Self {
        Self::with_services(AmmlServices::SIZE_V1)
    }

    /// A fake whose inventory table is v4: no `request_moves`, as a loader before `inventory.request`. Its
    /// `Inventory::request_moves` gives `NotAvailable`.
    ///
    /// # Panics
    /// When this thread already holds a live `FakeGame`.
    pub fn with_v4_inventory() -> Self {
        Self::with_tables(FakeTables::with_inventory(AmmlServices::SIZE_V8, amml_abi::AmmlInventoryTable::SIZE_V4))
    }

    fn with_services(size: u32) -> Self {
        Self::with_tables(FakeTables::new(size))
    }

    fn with_tables(tables: FakeTables) -> Self {
        let state = Rc::new(FakeState::new());
        if !FakeState::install(&state) {
            panic!("a FakeGame is already live on this thread; drop it first");
        }
        Self { state, tables }
    }

    /// The raw v3 table, as the loader passes it to `amml_mod_load`. It lives as long as the fake.
    pub fn api_table(&self) -> *const AmmlApi {
        self.tables.api()
    }

    /// A `Ctx` for mod `@fake/mod` that names no uses.
    pub fn ctx(&self) -> Ctx {
        self.ctx_with("@fake/mod", Uses::NONE)
    }

    /// A `Ctx` for the mod `module` (`@namespace/name`) that names no uses, for a test whose mod id
    /// matters, such as a link that compares it with its descriptor.
    pub fn ctx_named(&self, module: &'static str) -> Ctx {
        self.ctx_with(module, Uses::NONE)
    }

    /// A `Ctx` with the id and `USES` of `M`. Each call leaks the id string (`Ctx` holds a
    /// `&'static str`); a test makes few calls.
    pub fn ctx_for<M: AmmlMod>(&self) -> Ctx {
        let module: &'static str = Box::leak(M::id().into_boxed_str());
        self.ctx_with(module, M::USES)
    }

    fn ctx_with(&self, module: &'static str, uses: Uses) -> Ctx {
        // SAFETY: the tables are full v3 tables owned by `self`.
        match unsafe { Ctx::from_raw(self.tables.api(), module, uses) } {
            Ok(ctx) => ctx,
            Err(error) => panic!("the fake loader table was refused: {error}"),
        }
    }

    /// Makes `E` available at its version.
    pub fn offer<E: Event>(&self) {
        self.state.bus().offer(E::ID, E::VERSION);
    }

    /// Makes every later listen to `E` give `Refused`, as the loader does for a listen it will not
    /// take; `E` stays offered, so a required `E` passes the check before `enable`.
    pub fn refuse_listen<E: Event>(&self) {
        self.state.refuse_listen(E::ID);
    }

    /// Makes `area` available at its version.
    pub fn offer_area(&self, area: Area) {
        self.state.offer_area(area);
    }

    /// Takes `area` back, as the loader does when a later check finds it cannot serve it (an input
    /// feed off the game thread).
    pub fn withdraw_area(&self, area: Area) {
        self.state.withdraw_area(area);
    }

    /// A header for an `E` raised by the game.
    pub fn header<E: Event>() -> AmmlEventHeader {
        AmmlEventHeader {
            size: std::mem::size_of::<E>() as u32,
            version: E::VERSION,
            phase: AmmlPhase::BEFORE,
            origin: AmmlOrigin::GAME,
            cancelled: 0,
            injected: 0,
            _reserved: 0,
        }
    }

    /// Calls every listener of `E` with `value`, as the loader does: band order, a panicking listener
    /// is caught and its changes undone, and the header is written back before and after every
    /// listener call (`HeaderStamp`), so a listener cannot change it for later listeners; a cancel
    /// counts only for an `E::CANCELLABLE` event and cannot be undone. The header's `size`,
    /// `version` and `cancelled` are reset first. After a panic the mod is disabled as in the game:
    /// all its listeners are removed and new listens fail. The event carries no client.
    pub fn raise<E: Event + Clone>(&self, value: &mut E) -> FakeDispatch {
        self.dispatch(value, E::CANCELLABLE, std::ptr::null(), false)
    }

    /// `raise` with the fake client in the event context, once `set_client` ran (no client before).
    pub fn raise_with_client<E: Event + Clone>(&self, value: &mut E) -> FakeDispatch {
        let client = std::ptr::with_exposed_provenance(self.state.current_client());
        self.dispatch(value, E::CANCELLABLE, client, false)
    }

    // `blocked`: the event starts cancelled, as a real key that an active lease blocks.
    fn dispatch<E: Event + Clone>(
        &self,
        value: &mut E,
        cancellable: bool,
        client: *const c_void,
        blocked: bool,
    ) -> FakeDispatch {
        let header = Self::header_of(value);
        header.size = std::mem::size_of::<E>() as u32;
        header.version = E::VERSION;
        header.cancelled = u8::from(blocked);
        let mut stamp = HeaderStamp::new(*header, cancellable);
        let context = self.context(client);
        let report = self
            .state
            .bus()
            .dispatch(E::ID, value, |listener, value| Self::call(listener, value, &context, &mut stamp));
        stamp.stamp(Self::header_of(value));
        let panicked = !report.panicked_owners.is_empty();
        if panicked {
            self.state.disable_owner();
        }
        FakeDispatch { cancelled: report.cancelled, panicked }
    }

    // Raises an event a lease injected: `injected = 1`, the given origin, and a cancel is ignored.
    fn raise_injected<E: Event + Clone>(&self, value: &mut E, origin: AmmlOrigin) {
        let header = Self::header_of(value);
        header.injected = 1;
        header.origin = origin;
        let _ = self.dispatch(value, false, std::ptr::with_exposed_provenance(self.state.current_client()), false);
    }

    // The header at the start of `value`.
    fn header_of<E: Event>(value: &mut E) -> &mut AmmlEventHeader {
        // SAFETY: by the `Event` contract `E` starts with an `AmmlEventHeader`; the borrow of `value`
        // covers it.
        unsafe { &mut *std::ptr::from_mut(value).cast::<AmmlEventHeader>() }
    }

    fn call<E: Event>(
        listener: &FakeListener,
        value: &mut E,
        context: &AmmlEventContext,
        stamp: &mut HeaderStamp,
    ) -> CallOutcome {
        stamp.stamp(Self::header_of(value));
        let event = std::ptr::from_mut(value).cast::<AmmlEventHeader>();
        let user = std::ptr::with_exposed_provenance_mut(listener.user);
        // SAFETY: `event` points to a live `E` that starts with its header, `context` lives for the
        // call, and `user` is the pointer registered with this callback.
        let status = unsafe { (listener.callback)(event, context, user) };
        if status == AmmlStatus::PANICKED {
            return CallOutcome::Panicked;
        }
        if stamp.after_call(Self::header_of(value)) { CallOutcome::Cancelled } else { CallOutcome::Done }
    }

    /// Runs `M::load` as the loader does: `listen` and `provide` are accepted inside it only. A
    /// required listen whose event the fake does not offer fails the load.
    pub fn load<M: AmmlMod>(&self) -> Result<(), ApiError> {
        let result = self.registering(&self.ctx_for::<M>(), M::load);
        self.fail_on_error(result)
    }

    /// Runs `M::enable` as the loader does. The mod may listen and take leases again first (after a
    /// `disable`). A required event or area of `M::USES` that the fake does not offer fails before
    /// `enable` runs; a required listen whose event is not offered fails after it. A failure drops the
    /// mod's tables and listeners.
    pub fn enable<M: AmmlMod>(&self) -> Result<(), ApiError> {
        self.state.reopen_owner();
        self.state.input().reopen();
        let result = self.check_declared(M::USES).and_then(|()| self.registering(&self.ctx_for::<M>(), M::enable));
        self.fail_on_error(result)
    }

    /// Disables `M` in the loader's order: its tables are dropped (the fake has no other mod to
    /// raise `ProviderGone` to), its listeners are removed, then `M::disable` runs with
    /// `game_alive` from the gate; then its input leases end with no notice, and new leases are
    /// refused until the next `enable`.
    pub fn disable<M: AmmlMod>(&self) {
        self.state.drop_providers();
        self.state.disable_owner();
        self.state.phase().set(FakePhase::Running);
        let ctx = self.ctx_for::<M>();
        M::disable(&ctx, &ctx.disable_ctx(self.state.gate().get()));
        self.state.input().end_owner();
    }

    // Runs `callback` with listens and provides allowed, then refuses them again. A required listen
    // the call could not make fails it.
    fn registering(&self, ctx: &Ctx, callback: fn(&Ctx) -> Result<(), ApiError>) -> Result<(), ApiError> {
        let missing_before = self.state.missing_required().len();
        self.state.phase().set(FakePhase::Registering);
        let result = callback(ctx);
        self.state.phase().set(FakePhase::Running);
        result?;
        if self.state.missing_required().len() > missing_before {
            return Err(ApiError::NotAvailable);
        }
        Ok(())
    }

    // The first required event or area row of `uses` that the fake does not offer gives
    // `NotAvailable`.
    fn check_declared(&self, uses: Uses) -> Result<(), ApiError> {
        let missing = uses.rows().iter().any(|row| {
            // SAFETY: the rows of `Uses` point to static strings.
            let name = unsafe { row.name.as_str() };
            let offered = |name: &str| match row.kind {
                AmmlUseRow::KIND_EVENT => self.state.bus().is_available(name, row.version),
                AmmlUseRow::KIND_AREA => self.state.area_offered(name, row.version),
                _ => true,
            };
            row.required != 0 && name.is_ok_and(|name| !offered(name))
        });
        if missing { Err(ApiError::NotAvailable) } else { Ok(()) }
    }

    // A failed callback drops the mod's tables and listeners, as the loader does.
    fn fail_on_error(&self, result: Result<(), ApiError>) -> Result<(), ApiError> {
        if result.is_err() {
            self.state.drop_providers();
            self.state.disable_owner();
        }
        result
    }

    fn context(&self, client: *const c_void) -> AmmlEventContext {
        AmmlEventContext {
            size: std::mem::size_of::<AmmlEventContext>() as u32,
            version: 1,
            thread: AmmlThread::GAME,
            game_alive: u8::from(self.state.gate().get()),
            _reserved: [0; 6],
            epoch: self.state.epoch().get(),
            client,
        }
    }

    /// Opens the game gate, as the first `FrameUpdate` does: `game_alive` becomes true.
    pub fn open_gate(&self) {
        self.state.gate().set(true);
    }

    pub fn set_epoch(&self, epoch: u64) {
        self.state.epoch().set(epoch);
    }

    /// Makes `Dirs` give `path` for `kind`.
    pub fn set_dir(&self, kind: AmmlDirKind, path: &str) {
        self.state.set_dir(kind, path);
    }

    /// Every line the mod logged, as `<level> <message>` followed by ` key=value` per field.
    pub fn logs(&self) -> Vec<String> {
        self.state.logs()
    }

    /// The events a required listen asked for that the fake does not offer.
    pub fn missing_required(&self) -> Vec<String> {
        self.state.missing_required()
    }

    /// Sets the primary client: `raise_with_client` passes it, `DisableCtx::client` finds it, and the
    /// `Client` reads give `state` (each read also needs its area offered and the gate open).
    pub fn set_client(&self, state: FakeClient) {
        self.state.set_client(state);
    }

    /// Sets the inventory of the local player: the main-hand stack and the selected slot that `inventory.read`
    /// serves. Each read checks the gate, the client, its local player and the area as the loader does; a stack
    /// handle is accepted only while the fake still holds a stack.
    pub fn set_inventory(&self, inventory: crate::testing::FakeInventory) {
        self.state.set_inventory(inventory);
    }

    /// Sets the off-hand stack of the local player (`None` is an empty off hand), served by `held_item(Hand::Off)`.
    pub fn set_off_hand(&self, item: Option<crate::testing::FakeItem>) {
        self.state.lighting().set_off_hand(item);
    }

    /// Items named `type_name` place a block that gives `emission`: `ItemView::light_emission` gives it and
    /// `ItemView::block` a handle of the current world (`set_epoch`; none at epoch 0). Other items place no block.
    pub fn set_item_block(&self, type_name: &'static str, emission: u8) {
        self.state.lighting().set_item_block(type_name, emission);
    }

    /// The light the mod set last through `Lighting::set_light_source`; `None` before the first set or after a
    /// clear.
    pub fn light_source(&self) -> Option<crate::LightSource> {
        self.state.lighting().light().map(|light| crate::LightSource {
            position: crate::Vec3::from(light.position),
            level: light.level,
            falloff: if light.falloff == amml_abi::AmmlLightFalloff::TAXICAB {
                crate::LightFalloff::Taxicab
            } else {
                crate::LightFalloff::Round
            },
            block: if light.has_block != 0 { crate::BlockHandle::from_raw(light.block) } else { None },
        })
    }

    /// How many `set_light_source` calls the loader accepted.
    pub fn light_source_calls(&self) -> usize {
        self.state.lighting().calls()
    }

    /// Makes every `set_light_source` call give `Refused` (as the loader does off the game thread or for a fifth
    /// mod) while `refused`.
    pub fn refuse_lighting(&self, refused: bool) {
        self.state.lighting().set_refused(refused);
    }

    /// Raises `ScreenRender` for `view` of `screen`, as the loader does after the game drew the view: the ui table
    /// answers for `view` with the reads of `fake` while the listeners run, and refuses it after. The event
    /// carries tick 0. The event context holds the fake client once `set_client` ran, as the loader gives the primary
    /// client on the game thread.
    pub fn raise_screen_render(&self, screen: crate::ScreenId, view: crate::ViewId, fake: &FakeScreen) -> FakeDispatch {
        let mut event = amml_abi::ScreenRender {
            header: Self::header::<amml_abi::ScreenRender>(),
            screen: screen.0,
            view: view.0,
            tick: 0,
        };
        self.state.ui().enter(view.0, screen.0, fake.clone());
        let dispatch = self.raise_with_client(&mut event);
        self.state.ui().leave();
        dispatch
    }

    /// `raise_screen_render`, then the moves the listeners made in the dispatch (`screen_actions`) are kept in `fake`: its
    /// slots and cursor are the screen's after the dispatch, so the next render starts from them.
    /// `raise_screen_render` works on a copy and leaves `fake` as it was.
    pub fn raise_screen_render_mut(
        &self,
        screen: crate::ScreenId,
        view: crate::ViewId,
        fake: &mut FakeScreen,
    ) -> FakeDispatch {
        let mut event = amml_abi::ScreenRender {
            header: Self::header::<amml_abi::ScreenRender>(),
            screen: screen.0,
            view: view.0,
            tick: 0,
        };
        self.state.ui().enter(view.0, screen.0, fake.clone());
        let dispatch = self.raise_with_client(&mut event);
        if let Some(after) = self.state.ui().leave() {
            *fake = after;
        }
        dispatch
    }

    /// Raises `PlayerListUpdate` for `list`, as the loader does after the game handled the packet: the network table
    /// answers for the entries (at most 4096) while the listeners run, and refuses them after.
    pub fn raise_player_list(&self, list: &crate::testing::FakePlayerList) -> FakeDispatch {
        let entries = list.entries.clone();
        let count = entries.len().min(amml_abi::PlayerListUpdate::MAX_ENTRIES as usize);
        let mut event = amml_abi::PlayerListUpdate {
            header: Self::header::<amml_abi::PlayerListUpdate>(),
            action: list.action.into(),
            count: count as u32,
        };
        self.state.network().enter_list(list.action, entries);
        let dispatch = self.raise(&mut event);
        self.state.network().leave();
        dispatch
    }

    /// Raises `TexturesReloaded` with `reason`, as the loader does after it handled the mod textures: for
    /// `TextureReload::DeviceReset` every texture is freed first (a later use of its id gives `Stale`, as after a world
    /// leave); for `TextureReload::ResourcePack` the ids stay valid.
    pub fn raise_textures_reloaded(&self, reason: crate::TextureReload) -> FakeDispatch {
        let raw = match reason {
            crate::TextureReload::ResourcePack => amml_abi::AmmlTextureReload::RESOURCE_PACK,
            crate::TextureReload::DeviceReset => amml_abi::AmmlTextureReload::DEVICE_RESET,
            crate::TextureReload::Other => amml_abi::AmmlTextureReload(u8::MAX),
        };
        if reason == crate::TextureReload::DeviceReset {
            self.state.textures().leave_world();
        }
        let mut event = amml_abi::TexturesReloaded {
            header: Self::header::<amml_abi::TexturesReloaded>(),
            reason: raw,
            _reserved: [0; 7],
        };
        self.raise(&mut event)
    }

    /// Raises `PlayerSkinChanged` for the player `uuid` and its `skin`, as the loader does after the game handled the
    /// packet: the network table answers for the skin while the listeners run, and refuses it after.
    pub fn raise_skin(&self, uuid: crate::Uuid, skin: Option<crate::testing::FakeSkin>) -> FakeDispatch {
        let mut event = amml_abi::PlayerSkinChanged { header: Self::header::<amml_abi::PlayerSkinChanged>() };
        self.state.network().enter_skin(uuid, skin);
        let dispatch = self.raise(&mut event);
        self.state.network().leave();
        dispatch
    }

    /// The moves the container calls accepted so far, in order: `auto_place hotbar 3`, `take_all container 0`,
    /// `place_all inventory 5`, `drop_slot container 0 whole` (or `one` or a count), `drop_cursor one` and
    /// `swap container 0 inventory 4`. The fake applies each to the slots of the screen that runs (see `FakeScreen`).
    pub fn screen_actions(&self) -> Vec<String> {
        self.state.ui().actions()
    }

    /// Raises `ContainerSlotHover` for `slot` of `screen`, as the loader does after the game handled the hover.
    pub fn raise_hover(&self, screen: crate::ScreenId, slot: crate::SlotAt) -> FakeDispatch {
        let mut event = amml_abi::ContainerSlotHover {
            header: Self::header::<amml_abi::ContainerSlotHover>(),
            screen: screen.0,
            slot: slot.into(),
        };
        self.raise(&mut event)
    }

    /// Raises `ContainerSlotUnhover` for `slot` of `screen`.
    pub fn raise_unhover(&self, screen: crate::ScreenId, slot: crate::SlotAt) -> FakeDispatch {
        let mut event = amml_abi::ContainerSlotUnhover {
            header: Self::header::<amml_abi::ContainerSlotUnhover>(),
            screen: screen.0,
            slot: slot.into(),
        };
        self.raise(&mut event)
    }

    /// One pointer button of a UI screen, as the loader runs the detour on the game's button handler: raises
    /// `UiPointer` for `pointer` (0 or 1) of `view` at `x`, `y` with the input mode `Mouse`, then lets the up of a
    /// cancelled down stay cancelled (the event is raised already cancelled and its game copy is held back; a cancel of
    /// any other up is ignored). Gives whether the game gets the button event.
    pub fn ui_pointer(&self, view: crate::ViewId, pointer: u8, pressed: bool, x: f32, y: f32) -> bool {
        let gate = self.state.ui().pointers();
        let held = !pressed && gate.up(view.0, pointer);
        let mut event = amml_abi::UiPointer {
            header: Self::header::<amml_abi::UiPointer>(),
            screen: 0,
            view: view.0,
            pointer,
            pressed: u8::from(pressed),
            input_mode: amml_abi::AmmlInputMode::MOUSE.0,
            _reserved: 0,
            x,
            y,
            _reserved2: 0,
        };
        let client = std::ptr::with_exposed_provenance(self.state.current_client());
        let dispatch = self.dispatch(&mut event, <amml_abi::UiPointer as Event>::CANCELLABLE, client, held);
        if pressed { gate.down(view.0, pointer, dispatch.cancelled) } else { !held }
    }

    /// Raises `UiScroll` for one wheel step of the scene of `screen` (`None` when the loader does not know it).
    /// Gives whether a listener cancelled it, so the scene did not get the step.
    pub fn raise_scroll(&self, screen: Option<crate::ScreenId>, notches: i32) -> bool {
        let mut event = amml_abi::UiScroll {
            header: Self::header::<amml_abi::UiScroll>(),
            screen: screen.map_or(0, |screen| screen.0),
            notches,
            _reserved: 0,
        };
        self.raise(&mut event).cancelled
    }

    /// Raises `ItemHovertext` for a container item, as the loader does before the game builds the text. Gives the
    /// `contents_lines` after the listeners: 0 when one asked to hide the contents, else 1.
    pub fn raise_hovertext(&self) -> u8 {
        let mut event = amml_abi::ItemHovertext {
            header: Self::header::<amml_abi::ItemHovertext>(),
            kind: amml_abi::ItemHovertext::CONTAINER_ITEM,
            contents_lines: 1,
            _reserved: [0; 3],
        };
        self.raise(&mut event);
        event.contents_lines
    }

    /// Raises `LocatorBarIconDraw` for a waypoint that follows `actor`, as the loader does before the game blits the
    /// icon: the rect is 10, 20 and 8 by 9, the colour is white and there is no texture. Gives what the game draws
    /// after the listeners, by the loader's rules: a texture that is unknown, stale or not ready drops every edit; a
    /// rect with a value that is not finite is not applied.
    pub fn raise_locator_icon(&self, actor: Option<crate::ActorUniqueId>) -> crate::testing::FakeIconEdit {
        let rect = crate::Rect { x: 10.0, y: 20.0, width: 8.0, height: 9.0 };
        let white = crate::Rgba { r: 255, g: 255, b: 255, a: 255 };
        let mut event = amml_abi::LocatorBarIconDraw {
            header: Self::header::<amml_abi::LocatorBarIconDraw>(),
            waypoint_actor_id: actor.map_or(0, |actor| actor.0),
            has_actor: u8::from(actor.is_some()),
            _reserved: [0; 7],
            rect: [rect.x, rect.y, rect.width, rect.height],
            uv: [0.0, 1.0, 0.0, 1.0],
            light: 1.0,
            alpha: 1.0,
            color: 0xFFFF_FFFF,
            second_color: 0xFF00_0000,
            scale: [1.0, 1.0],
            texture: amml_abi::AmmlTextureId::default(),
        };
        self.raise(&mut event);
        let own = crate::testing::FakeIconEdit { texture: None, rect, color: white };
        let texture = (event.texture.id != 0).then(|| crate::TextureId::from(event.texture));
        if texture.is_some_and(|texture| !self.state.textures().is_ready(texture)) {
            return own;
        }
        let [x, y, width, height] = event.rect;
        let edited = crate::Rect { x, y, width, height };
        let [a, r, g, b] = event.color.to_be_bytes();
        crate::testing::FakeIconEdit {
            texture,
            rect: if event.rect.iter().all(|value| value.is_finite()) { edited } else { rect },
            color: crate::Rgba { r, g, b, a },
        }
    }

    /// The requests `Inventory::request_moves` took so far, in order: the id, the moves and the options. The fake
    /// runs one at a time, as the loader does: a request runs until `answer` and the next `frame` end it.
    pub fn requests(&self) -> Vec<(crate::RequestId, Vec<crate::InventoryMove>, crate::RequestOptions)> {
        self.state.requests().all()
    }

    /// The server answers request `id`: `accepted` of its moves were accepted (all of them gives `Done`, fewer
    /// gives `Refused`). The next `frame` raises `InventoryRequestDone`, never this call. False for an id that does
    /// not run.
    pub fn answer(&self, id: crate::RequestId, accepted: u16) -> bool {
        self.state.requests().answer(id, accepted)
    }

    /// The draw calls the ui table accepted so far, in order: `fill <x> <y> <width> <height>`, `outline ...` and
    /// `image <texture id> <x> <y> <width> <height>`, `item <type name> <x> <y> <flags>` and `text "<text>" <x> <y>`.
    /// The fake font is 6 units wide a character; the fake does not count the texts of one dispatch.
    pub fn draws(&self) -> Vec<String> {
        self.state.ui().draws()
    }

    /// The item draw handles that are not freed yet: the live ones and the ones dropped on another thread that
    /// `pump_item_draws` has not freed.
    pub fn live_item_draws(&self) -> usize {
        self.state.items().held()
    }

    /// Applies the frees of item draw handles dropped on other threads, as the loader does at its next UI draw.
    /// Gives the number of handles freed.
    pub fn pump_item_draws(&self) -> usize {
        self.state.items().pump()
    }

    /// A world leave for the item draws: every handle is freed, and a later use of one gives `Stale`.
    pub fn leave_item_world(&self) {
        self.state.items().leave_world();
    }

    /// Raises `ChatMessageAdded` for `chat`, as the loader does after the game built the line: the reads of the event
    /// give the line as it is, and the edits wait until the last listener returned. Gives the text and the filtered text
    /// after the edits (the latest edit of a field wins; a filtered edit of a line with no filtered text is refused).
    pub fn raise_chat(&self, chat: &crate::testing::FakeChat) -> (Option<String>, Option<String>) {
        let mut event = amml_abi::ChatMessageAdded {
            header: Self::header::<amml_abi::ChatMessageAdded>(),
            kind: chat.kind.0,
            _reserved: 0,
        };
        self.state.chat().enter(chat);
        self.raise(&mut event);
        self.state.chat().leave()
    }

    /// Sets whether the default font is a bitmap font. `Chat::register_inline_icon` gives `NotAvailable` and
    /// `ChatMessageAddedExt::inline_icons` is false while it is not. A fake starts with no bitmap font.
    pub fn set_bitmap_font(&self, bitmap: bool) {
        self.state.chat().set_bitmap(bitmap);
    }

    /// The characters of the inline icons that are not freed yet, in order: the live ones and the ones dropped on another
    /// thread that `pump_icons` has not freed.
    pub fn icons(&self) -> Vec<char> {
        self.state.chat().glyphs()
    }

    /// Applies the frees of icons dropped on other threads, as the loader does at its next UI draw. Gives the number of
    /// icons freed.
    pub fn pump_icons(&self) -> usize {
        self.state.chat().pump()
    }

    /// A world leave for the inline icons: every icon is freed, and a later use of one gives `Stale`.
    pub fn leave_icon_world(&self) {
        self.state.chat().leave_world();
    }

    /// The textures of the fake: what the mod asked for, the ready flag, the world leave and the pump of frees made
    /// on other threads.
    pub fn textures(&self) -> crate::testing::FakeTextures {
        crate::testing::FakeTextures::new(self.state.clone())
    }

    /// Raises `ScreenClosing` for `screen`, as the loader does before the game frees its controller.
    pub fn raise_screen_closing(&self, screen: crate::ScreenId) -> FakeDispatch {
        let mut event = amml_abi::ScreenClosing { header: Self::header::<amml_abi::ScreenClosing>(), screen: screen.0 };
        self.raise(&mut event)
    }

    /// Raises `ScreenChanged` from screen `old` to screen `new`, with the top screen name `screen`.
    pub fn raise_screen_changed(
        &self,
        old: crate::ScreenId,
        new: crate::ScreenId,
        screen: crate::Screen,
    ) -> FakeDispatch {
        let mut event = amml_abi::ScreenChanged {
            header: Self::header::<amml_abi::ScreenChanged>(),
            old: old.0,
            new: new.0,
            screen: amml_abi::AmmlScreen(screen as u8),
            _reserved: [0; 7],
        };
        self.raise_with_client(&mut event)
    }

    /// Raises `JoinServerResult` for the join `join` with `outcome`, as the loader does at the frame after the game's
    /// answer (to the listeners of the mod; the fake has one mod).
    pub fn raise_join_result(&self, join: crate::JoinId, outcome: amml_abi::AmmlJoinOutcome) -> FakeDispatch {
        let mut event = amml_abi::JoinServerResult {
            header: Self::header::<amml_abi::JoinServerResult>(),
            join: join.0,
            outcome,
            _reserved: 0,
        };
        self.raise(&mut event)
    }

    /// Every client write in order: `hide_hud=true`, `mouse_captured=false`, `open_pause_screen`,
    /// `leave_world`, `start_local_world <id>`, `join_server <host>:<port>`, `open_menu servers`, `send_command <text>`,
    /// `select_hotbar_slot <slot>`.
    pub fn client_writes(&self) -> Vec<String> {
        self.state.client_writes()
    }

    /// Attaches the game window: the `Window` reads give `window` (each also needs `window.read`
    /// offered), and `request_size` and `replay_focus_loss` are accepted. Until then they give `None` or
    /// `NotReady`.
    pub fn set_window(&self, window: FakeWindow) {
        self.state.set_window(window);
    }

    /// Every accepted window request in order: `request_size 640x360`, `request_size none`,
    /// `replay_focus_loss`.
    pub fn window_requests(&self) -> Vec<String> {
        self.state.window_requests()
    }

    /// Every accepted screenshot request in order. The fake writes no file and raises no
    /// `ScreenshotTaken` or `ScreenshotWritten`; a test raises them with the id it got.
    pub fn screenshots(&self) -> Vec<(PathBuf, ShotLayers)> {
        self.state.overlay().screenshots()
    }

    /// One present of the game's swap chain: `Overlay::present_count` goes up by one, and each open
    /// capture stream for which it is an nth present gets the frame (`width x height`, `bgra` with
    /// `width * 4` bytes per row) in a free slot, or counts a skip. The frame's `present_index` is the new
    /// present count.
    pub fn push_capture_frame(&self, width: u32, height: u32, bgra: Vec<u8>) {
        self.state.overlay().present(width, height, &bgra);
    }

    /// The capture streams open now; a dropped `CaptureStream` is closed.
    pub fn open_capture_streams(&self) -> usize {
        self.state.overlay().open_streams()
    }

    /// The presents the capture streams skipped because no slot was free, summed over the streams.
    pub fn capture_skipped(&self) -> u64 {
        self.state.overlay().skipped()
    }

    /// Binds every input feed, as the loader does at the first real feed of each kind. Until then an
    /// injection gives `NotReady`.
    pub fn bind_input(&self) {
        self.state.input().bind();
    }

    /// What reached the game, in order: `key 0x57 down`, `key 0x57 up`, `button left down`, `wheel 1`,
    /// `turn 90 0` (the summed turn of a frame, after the `LookDelta` listeners), `move forward=1
    /// jump` (the axes that are not 0 and the held actions; `move none` when cleared). Input reaches
    /// the game only at `frame`.
    pub fn injected(&self) -> Vec<String> {
        self.state.input().injected()
    }

    /// One key from the user's keyboard, as the loader runs a real key feed: the key table records it;
    /// an up of a key the fake saw no down for reaches the game and raises nothing; any other key raises
    /// `KeyInput` (with the fake client), already cancelled when an active lease blocks `KEYS` and no
    /// passthrough names the key; then the key table decides from the cancel whether the game gets it (a
    /// cancelled first down holds back its repeats and its up, a cancel of an up whose down reached the
    /// game is ignored). Gives whether the game gets the key. Real keys show in `Input::is_down` and
    /// `modifiers`, not in `injected`.
    pub fn real_key(&self, key: u16, down: bool) -> bool {
        let input = self.state.input();
        let verdict = input.real_key(key, down);
        if !verdict.raise {
            return true;
        }
        let mut event = KeyInput {
            header: Self::header::<KeyInput>(),
            key,
            down: u8::from(down),
            repeat: u8::from(verdict.repeat),
            modifiers: input.modifiers(),
        };
        let client = std::ptr::with_exposed_provenance(self.state.current_client());
        let _ = self.dispatch(&mut event, <KeyInput as Event>::CANCELLABLE, client, input.blocks_key(key));
        input.finish_real_key(key, down, event.header.cancelled != 0)
    }

    /// Runs the input part of a frame, as the loader does at `FrameUpdate` Before: the lease notices
    /// since the last frame are raised (`InputLeaseSuspended`, `InputLeaseResumed`, `InputLeaseLost`,
    /// `InjectedInputReleased`), then the queued keys, buttons and wheel notches go through the key
    /// and button tables and each one that reaches the game raises an injected `KeyInput` or
    /// `MouseInput`, then the summed turn raises `LookDelta`, and a changed move intent is recorded.
    /// It raises no `FrameUpdate`. A key down that is released before a frame reaches the game as
    /// nothing.
    pub fn frame(&self) {
        for (id, outcome, accepted, total) in self.state.requests().take_ended() {
            let mut event = InventoryRequestDone {
                header: Self::header::<InventoryRequestDone>(),
                id: id.0,
                outcome: outcome.0,
                _reserved: 0,
                accepted,
                total,
                _reserved2: 0,
            };
            let _ = self.raise(&mut event);
        }
        let input = self.state.input();
        for (lease, kind) in input.take_notices() {
            self.raise_notice(lease, kind);
        }
        let frame = input.take_frame();
        for (lease, op) in frame.ops {
            match op {
                InjectedOp::Key { key, down } => {
                    if !input.key_reaches_game(lease, key, down) {
                        continue;
                    }
                    let mut event = KeyInput {
                        header: Self::header::<KeyInput>(),
                        key,
                        down: u8::from(down),
                        repeat: 0,
                        modifiers: input.modifiers(),
                    };
                    self.raise_injected(&mut event, AmmlOrigin::THIS_MOD);
                    input.record(format!("key 0x{key:02X} {}", Self::updown(down)));
                }
                InjectedOp::Button { button, down } => {
                    if !input.button_reaches_game(lease, button, down) {
                        continue;
                    }
                    let mut event = Self::mouse_event(AmmlMouseAction::BUTTON, button, down, 0.0);
                    self.raise_injected(&mut event, AmmlOrigin::THIS_MOD);
                    input.record(format!("button {} {}", Self::button_name(button), Self::updown(down)));
                }
                InjectedOp::Wheel { notches } => {
                    let notch = if notches < 0.0 { -1.0 } else { 1.0 };
                    for _ in 0..(notches.abs().trunc() as u32).min(Self::MAX_WHEEL_NOTCHES) {
                        let mut event = Self::mouse_event(AmmlMouseAction::WHEEL, AmmlMouseButton::LEFT, false, notch);
                        self.raise_injected(&mut event, AmmlOrigin::THIS_MOD);
                        input.record(format!("wheel {notch}"));
                    }
                }
            }
        }
        if let Some((yaw, pitch)) = frame.turn {
            let mut event = LookDelta { header: Self::header::<LookDelta>(), yaw, pitch };
            self.raise_injected(&mut event, AmmlOrigin::OTHER_MOD);
            if event.yaw.is_finite() && event.pitch.is_finite() {
                input.record(format!("turn {} {}", event.yaw, event.pitch));
            }
        }
        let intent = frame.move_intent.map(|(_, intent)| intent);
        if input.move_changed(intent) {
            input.record(Self::move_line(intent));
        }
    }

    fn raise_notice(&self, lease: u64, kind: NoticeKind) {
        match kind {
            NoticeKind::Suspended => {
                let _ = self.raise(&mut InputLeaseSuspended { header: Self::header::<InputLeaseSuspended>(), lease });
            }
            NoticeKind::Resumed => {
                let _ = self.raise(&mut InputLeaseResumed { header: Self::header::<InputLeaseResumed>(), lease });
            }
            NoticeKind::Lost(reason) => {
                let _ = self.raise(&mut InputLeaseLost { header: Self::header::<InputLeaseLost>(), lease, reason });
            }
            NoticeKind::Released(reason) => {
                let header = Self::header::<InjectedInputReleased>();
                let input = InjectedInputReleased::ALL;
                let _ = self.raise(&mut InjectedInputReleased { header, lease, reason, input, code: 0, _reserved: 0 });
            }
            NoticeKind::ReleasedOne { reason, input, code } => {
                let header = Self::header::<InjectedInputReleased>();
                let _ = self.raise(&mut InjectedInputReleased { header, lease, reason, input, code, _reserved: 0 });
            }
        }
    }

    fn mouse_event(action: AmmlMouseAction, button: AmmlMouseButton, down: bool, notches: f32) -> MouseInput {
        MouseInput {
            header: Self::header::<MouseInput>(),
            action,
            button,
            down: u8::from(down),
            _reserved: 0,
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            notches,
        }
    }

    fn updown(down: bool) -> &'static str {
        if down { "down" } else { "up" }
    }

    fn button_name(button: AmmlMouseButton) -> &'static str {
        match button {
            AmmlMouseButton::LEFT => "left",
            AmmlMouseButton::RIGHT => "right",
            AmmlMouseButton::MIDDLE => "middle",
            AmmlMouseButton::X1 => "x1",
            _ => "x2",
        }
    }

    fn move_line(intent: Option<AmmlMoveIntent>) -> String {
        let Some(intent) = intent else {
            return "move none".to_string();
        };
        let mut line = "move".to_string();
        for (name, value) in [("forward", intent.forward), ("left", intent.left)] {
            if value != 0.0 {
                line.push_str(&format!(" {name}={value}"));
            }
        }
        for (name, held) in [("jump", intent.jump), ("sneak", intent.sneak), ("sprint", intent.sprint)] {
            if held != 0 {
                line.push(' ');
                line.push_str(name);
            }
        }
        line
    }
}

impl Drop for FakeGame {
    fn drop(&mut self) {
        FakeState::uninstall(&self.state);
    }
}

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

use amml_abi::{
    AmmlEvent, AmmlEventContext, AmmlInputMode, AmmlPhase, AmmlPoint, AmmlRect, AmmlScreen, AmmlScreenKind, AmmlSize,
    AmmlSlotAt, AmmlStatus, AmmlThread, AmmlUiProfile, ContainerSlotHover, ContainerSlotUnhover, ScreenChanged,
    ScreenClosing, ScreenRender,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcString;
use amml_runtime::{CollectionNames, PanelNames, ScreenBook, TopScreenWatch, UiScopeKind};

use crate::{
    ClientAdapter, ContainerAdapter, DispatchFinish, EventHub, InventoryAdapter, LoaderState, ObjectRead,
    ScreenControllerDetours, ScreenDetours, ScreenLayout, SessionAdapter, ThreadNote, UiScope, UiTreeWalk,
};

static ADAPTER: OnceLock<ScreenAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// `UIProfile IOptionRegistry::getUIProfile() const`: rcx the registry, the enum in eax.
type ProfileFn = unsafe extern "C" fn(usize) -> i32;

/// The screen lifecycle: `ui.screen_render`, `ui.screen_closing`, `ui.screen_changed`, the container hover events
/// (`ui.container_slot_hover` and `ui.container_slot_unhover`, raised after the game's hover handlers) and the reads of
/// `ui.screen`.
///
/// The detour on `ScreenView.render` gives the view and its controller ids (`ScreenBook`), notes the last view drawn
/// for the top screen (`TopScreenWatch`) and, while a `ScreenRender` listener exists, raises the event inside a
/// `UiScope`. The `ui` table answers only for the view of that running scope. With no listener of `ScreenRender`
/// or `ScreenChanged` the detour does one relaxed load (`UiFlags`) and returns.
pub struct ScreenAdapter {
    state: &'static LoaderState,
    book: ScreenBook,
    watch: TopScreenWatch,
    layout: Option<ScreenLayout>,
    walk: Option<UiTreeWalk>,
    // The client ticks counted so far.
    tick: AtomicU64,
    // The UI profile read on the game thread at the last client tick; `PROFILE_UNKNOWN` before the first.
    profile: AtomicU8,
    render_thread: ThreadNote,
    hover_thread: ThreadNote,
    // Per screen: the game frame of its last draw, and whether the order of a hover against the draw was logged.
    order: Mutex<HashMap<u64, (u64, [bool; 2])>>,
    unhover_thread: ThreadNote,
    // The top screen value of the previous client tick, for `screen draw order`; `u8::MAX` before the first.
    order_screen: AtomicU8,
    // The `screen draw order` lines logged so far.
    order_lines: AtomicU8,
}

impl ScreenAdapter {
    pub const SCREEN: &'static str = "ui.screen";
    pub const SCREEN_RENDER: &'static str = ScreenRender::ID;
    pub const SCREEN_CLOSING: &'static str = ScreenClosing::ID;
    pub const SCREEN_CHANGED: &'static str = ScreenChanged::ID;
    pub const CONTAINER_SLOT_HOVER: &'static str = ContainerSlotHover::ID;
    pub const CONTAINER_SLOT_UNHOVER: &'static str = ContainerSlotUnhover::ID;
    const AREA_VERSION: u32 = 1;
    // The version of `ui.screen` that adds the container calls.
    const CONTAINER_VERSION: u32 = 2;
    const PROFILE_UNKNOWN: u8 = u8::MAX;
    // At most this many `screen draw order` lines per process.
    const MAX_ORDER_LINES: u8 = 8;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the view draw and the controller destructor for the rows whose catalog shape resolved, offers each
    /// event and the `ui.screen` area whose hooks are in, and logs one line per row. Works once per process: a
    /// second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the screen adapter is already installed");
            return rows;
        }
        let shaped = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let render_wanted = [
            Self::SCREEN_RENDER,
            Self::SCREEN_CHANGED,
            Self::SCREEN,
            Self::CONTAINER_SLOT_HOVER,
            Self::CONTAINER_SLOT_UNHOVER,
        ]
        .into_iter()
        .any(shaped);
        let render_on = render_wanted && ScreenDetours::render_hook().install(state, Self::PRIORITY);
        let closing_on = shaped(Self::SCREEN_CLOSING) && ScreenControllerDetours::install(state, Self::PRIORITY);
        let hover_on = shaped(Self::CONTAINER_SLOT_HOVER) && ScreenDetours::hover_hook().install(state, Self::PRIORITY);
        let unhover_on =
            shaped(Self::CONTAINER_SLOT_UNHOVER) && ScreenDetours::unhover_hook().install(state, Self::PRIORITY);
        let layout = ScreenLayout::new(state);
        let walk = UiTreeWalk::new(state);
        let ids_on = render_on && layout.is_some();
        let reads_on = ids_on && walk.is_some();
        // The container calls need the reads of the view; without their entries `ui.screen` is still served and the
        // container calls give `NOT_AVAILABLE`.
        if reads_on && ContainerAdapter::install(state) {
            curia::info!("container calls installed", { area: Self::SCREEN });
        }
        let adapter = Self {
            state,
            book: ScreenBook::new(),
            watch: TopScreenWatch::new(),
            layout,
            walk,
            tick: AtomicU64::new(0),
            profile: AtomicU8::new(Self::PROFILE_UNKNOWN),
            render_thread: ThreadNote::new(),
            hover_thread: ThreadNote::new(),
            order: Mutex::new(HashMap::new()),
            unhover_thread: ThreadNote::new(),
            order_screen: AtomicU8::new(u8::MAX),
            order_lines: AtomicU8::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the screen adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            let on = match row.id {
                Self::SCREEN_RENDER => render_on,
                // The top screen is matched by the root control name of the drawn views, a read of the view.
                Self::SCREEN_CHANGED => reads_on,
                Self::SCREEN_CLOSING => closing_on,
                Self::CONTAINER_SLOT_HOVER => hover_on && ids_on,
                Self::CONTAINER_SLOT_UNHOVER => unhover_on && ids_on,
                _ => reads_on,
            };
            if row.shape.is_some() && !on {
                row.shape = None;
                row.missing = vec!["a hook or a read of the screen view"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Event) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (Some(shape), NeedKind::Area) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (None, NeedKind::Event) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
                (None, NeedKind::Area) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static ScreenAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ScreenAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        [
            Self::SCREEN_RENDER,
            Self::SCREEN,
            Self::SCREEN_CLOSING,
            Self::SCREEN_CHANGED,
            Self::CONTAINER_SLOT_HOVER,
            Self::CONTAINER_SLOT_UNHOVER,
        ]
        .contains(&row.id)
    }

    /// The ids of the controllers and views.
    pub fn book(&self) -> &ScreenBook {
        &self.book
    }

    // The context of an event raised on this thread: the game thread when it is that, else the render thread,
    // with the primary client on the game thread.
    fn context(&self, client: Option<usize>) -> AmmlEventContext {
        Self::context_of(self.state, client)
    }

    // `context` for an adapter that holds no screen adapter.
    pub(crate) fn context_of(state: &LoaderState, client: Option<usize>) -> AmmlEventContext {
        let hub = state.events();
        let game = hub.gate().on_game_thread();
        let thread = if game { AmmlThread::GAME } else { AmmlThread::RENDER };
        let client = client.or_else(|| if game { SessionAdapter::primary_client() } else { None }).unwrap_or(0);
        AmmlEventContext { client: std::ptr::with_exposed_provenance::<c_void>(client), ..hub.context_on(thread) }
    }

    /// After the game drew `view`: gives the ids, notes the view for the top screen and raises `ScreenRender`
    /// inside a `UiScope` while a mod listens to it.
    pub(crate) fn rendered(&self, view: *mut c_void, context: *mut c_void) {
        let Some(layout) = &self.layout else { return };
        let view_address = view.expose_provenance();
        // SAFETY: `view` is the live `ScreenView` the game passed to its own `render`, whose `mController`
        // member holds the controller pointer.
        let Some(controller) = (unsafe { ObjectRead::pointer(view_address, layout.controller) }) else { return };
        let Some(screen) = self.dispatch_view(controller, view_address, context.expose_provenance()) else { return };
        let Some(walk) = &self.walk else { return };
        // SAFETY: `view` is the live view inside its own `render` call on this thread.
        if let Some(name) = unsafe { walk.root_name(view_address) } {
            self.watch.set_name(screen, &name);
        }
    }

    /// Gives the ids of `controller` and the view at `view_address`, notes the view for the top screen and raises
    /// `ScreenRender` inside a `UiScope` whose draw calls go to the render context at `context`, while a mod listens
    /// to it. After the last listener the images the listeners queued are drawn. Gives the screen id when the screen
    /// has no root control name yet, so the caller reads it from the live view.
    pub fn dispatch_view(&self, controller: usize, view_address: usize, context: usize) -> Option<u64> {
        let (screen, view_id) = self.book.note(controller, view_address);
        let unnamed = self.watch.note_drawn(screen).then_some(screen);
        if ScreenDetours::hover_wanted() {
            self.order_lock().entry(screen).or_default().0 = InventoryAdapter::frame();
        }
        let hub = self.state.events();
        if !hub.has_listeners(ScreenRender::ID) {
            return unnamed;
        }
        let event_context = self.context(None);
        if self.render_thread.changed(event_context.thread) {
            curia::info!("ui thread", { hook: ScreenDetours::RENDER, game: event_context.thread == AmmlThread::GAME });
        }
        let mut event = ScreenRender {
            header: EventHub::header::<ScreenRender>(AmmlPhase::AFTER),
            screen,
            view: view_id,
            tick: self.tick.load(Ordering::Relaxed),
        };
        let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: view_id }, context);
        let _finish = DispatchFinish::new(context);
        hub.raise(&mut event, &event_context);
        unnamed
    }

    /// Before the game frees `controller`: retires its ids and raises `ScreenClosing` for a screen the book gave
    /// an id.
    pub(crate) fn closing(&self, controller: *mut c_void) {
        let Some(screen) = self.book.retire(controller.expose_provenance()) else { return };
        self.order_lock().remove(&screen);
        self.watch.forget(screen);
        let hub = self.state.events();
        if !hub.has_listeners(ScreenClosing::ID) {
            return;
        }
        let mut event = ScreenClosing { header: EventHub::header::<ScreenClosing>(AmmlPhase::BEFORE), screen };
        hub.raise(&mut event, &self.context(None));
    }

    /// At `ClientTick` of the primary `client`, on the game thread: counts the tick, then raises `ScreenChanged`
    /// when the top screen changed. The top screen is the drawn view whose root control name is the game's top
    /// screen name (`TopScreenWatch`); a toast is no change.
    pub(crate) fn client_tick(&self, client: usize) {
        self.tick.fetch_add(1, Ordering::Relaxed);
        self.cache_profile();
        let hub = self.state.events();
        if !hub.has_listeners(ScreenChanged::ID) {
            return;
        }
        let Some(adapter) = ClientAdapter::get() else { return };
        let handle = std::ptr::with_exposed_provenance::<c_void>(client);
        let Ok((screen, name)) = adapter.top_screen_named(handle) else { return };
        let change = self.watch.tick(&name, screen == AmmlScreen::TOAST);
        self.note_draw_order(screen, change.is_some());
        let Some(change) = change else { return };
        let mut event = ScreenChanged {
            header: EventHub::header::<ScreenChanged>(AmmlPhase::AFTER),
            old: change.old,
            new: change.new,
            screen,
            _reserved: [0; 7],
        };
        hub.raise(&mut event, &self.context(Some(client)));
    }

    // Logs `screen draw order` at a tick whose top screen value differs from the previous tick's, up to
    // `MAX_ORDER_LINES` times: the screens drawn in the tick, in the order of their last draw, each with its root
    // control name, and whether a `ScreenChanged` was raised.
    fn note_draw_order(&self, screen: AmmlScreen, changed: bool) {
        if self.order_screen.swap(screen.0, Ordering::Relaxed) == screen.0 {
            return;
        }
        let lines = self.order_lines.load(Ordering::Relaxed);
        if lines >= Self::MAX_ORDER_LINES {
            return;
        }
        self.order_lines.store(lines + 1, Ordering::Relaxed);
        let drawn: Vec<String> = self
            .watch
            .drawn_order()
            .into_iter()
            .map(|id| format!("{id} {}", self.watch.name_of(id).unwrap_or_default()))
            .collect();
        curia::info!("screen draw order", { screen: format!("{screen:?}"), drawn: drawn.join(", "), changed: changed });
    }

    // The address of the live view with id `view`, when a table call may read it: the adapter serves the area
    // (`NOT_AVAILABLE`), the game gate is open (`NOT_READY`), and the innermost running scope of this thread is the
    // `ScreenRender` dispatch of a live `view` (`REFUSED`).
    fn enter(&self, view: u64) -> Result<usize, AmmlStatus> {
        self.enter_at(view, Self::AREA_VERSION)
    }

    // `enter` for a call that needs the area at `version`.
    fn enter_at(&self, view: u64, version: u32) -> Result<usize, AmmlStatus> {
        if !self.state.areas().is_available(Self::SCREEN, version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.state.events().gate().require_open()?;
        if !UiScope::runs_view(view) {
            return Err(AmmlStatus::REFUSED);
        }
        self.book.view_address(view).ok_or(AmmlStatus::REFUSED)
    }

    fn reads(&self) -> Result<(&ScreenLayout, &UiTreeWalk), AmmlStatus> {
        match (&self.layout, &self.walk) {
            (Some(layout), Some(walk)) => Ok((layout, walk)),
            _ => Err(AmmlStatus::NOT_AVAILABLE),
        }
    }

    /// The kind of the screen of `view`; `OTHER` when its tree cannot be read.
    pub(crate) fn view_kind(&self, view: u64) -> Result<AmmlScreenKind, AmmlStatus> {
        let address = self.enter(view)?;
        let (_, walk) = self.reads()?;
        // SAFETY: `enter` proved `address` is the live view of a `ScreenRender` dispatch running on this thread.
        Ok(unsafe { walk.kind(address) }.unwrap_or(AmmlScreenKind::OTHER))
    }

    /// The size of `view` in GUI units, rounded down.
    pub(crate) fn view_size(&self, view: u64) -> Result<AmmlSize, AmmlStatus> {
        let address = self.enter(view)?;
        let (layout, _) = self.reads()?;
        // SAFETY: `enter` proved `address` is the live view; `mSize` is two f32.
        let size = unsafe { ObjectRead::value::<[f32; 2]>(address, layout.size) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if !size.iter().all(|value| value.is_finite()) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Ok(AmmlSize { width: size[0].floor() as u32, height: size[1].floor() as u32 })
    }

    /// The content rectangle of `view`: the panel `PanelNames` gives for its kind and the UI profile.
    /// `NOT_AVAILABLE` for a screen without one and while the panel is hidden.
    pub(crate) fn view_content_rect(&self, view: u64) -> Result<AmmlRect, AmmlStatus> {
        let address = self.enter(view)?;
        let (_, walk) = self.reads()?;
        // SAFETY: `enter` proved `address` is the live view of a `ScreenRender` dispatch running on this thread.
        let kind = unsafe { walk.kind(address) }.unwrap_or(AmmlScreenKind::OTHER);
        let profile = self.profile().unwrap_or(AmmlUiProfile::CLASSIC);
        let path = PanelNames::path(kind, profile).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: as above.
        unsafe { walk.panel(address, path) }.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The last pointer position of `view`, in GUI units.
    pub(crate) fn view_pointer(&self, view: u64) -> Result<AmmlPoint, AmmlStatus> {
        let address = self.enter(view)?;
        let (layout, _) = self.reads()?;
        // SAFETY: `enter` proved `address` is the live view; the pointer member is two f32.
        let at = unsafe { ObjectRead::value::<[f32; 2]>(address, layout.pointer) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if !at.iter().all(|value| value.is_finite()) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(AmmlPoint { x: at[0], y: at[1] })
    }

    /// The input mode of `view`; a value that is not one of the game's four is `UNKNOWN`.
    pub(crate) fn view_input_mode(&self, view: u64) -> Result<AmmlInputMode, AmmlStatus> {
        let address = self.enter(view)?;
        let (layout, _) = self.reads()?;
        // SAFETY: `enter` proved `address` is the live view; the input mode member is a 32-bit enum.
        let mode = unsafe { ObjectRead::value::<u32>(address, layout.input_mode) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok(match mode {
            1 => AmmlInputMode::MOUSE,
            2 => AmmlInputMode::TOUCH,
            3 => AmmlInputMode::GAME_PAD,
            4 => AmmlInputMode::MOTION_CONTROLLER,
            _ => AmmlInputMode::UNKNOWN,
        })
    }

    /// The UI profile option of the primary client.
    pub(crate) fn view_ui_profile(&self, view: u64) -> Result<AmmlUiProfile, AmmlStatus> {
        self.enter(view)?;
        self.profile()
    }

    /// The screen id of `view` when its controller is a container screen, else 0.
    pub(crate) fn view_container(&self, view: u64) -> Result<u64, AmmlStatus> {
        self.enter(view)?;
        let (layout, _) = self.reads()?;
        let controller = self.book.controller_of_view(view).ok_or(AmmlStatus::REFUSED)?;
        if !ClientAdapter::call_bool(controller, Some(layout.container_slot))? {
            return Ok(0);
        }
        self.book.screen_of_view(view).ok_or(AmmlStatus::REFUSED)
    }

    // The order map holds plain data, so a lock poisoned by a panic is recovered.
    fn order_lock(&self) -> std::sync::MutexGuard<'_, HashMap<u64, (u64, [bool; 2])>> {
        self.order.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // Logs once per screen whether a hover came before or after the draw of its view in the same game frame.
    fn note_order(&self, screen: u64) {
        let frame = InventoryAdapter::frame();
        let mut order = self.order_lock();
        let entry = order.entry(screen).or_default();
        let before = entry.0 < frame;
        if !std::mem::replace(&mut entry.1[usize::from(before)], true) {
            if before {
                curia::info!("hover before render", { screen: screen, frame: frame });
            } else {
                curia::info!("hover after render", { screen: screen, frame: frame });
            }
        }
    }

    /// The controller and the screen kind of `view` when a container call may use it: `ui.screen` at version 2 is
    /// served, the gate is open, the innermost running scope of this thread is the `ScreenRender` dispatch of the
    /// live `view`, and its controller is a container screen (`REFUSED` when it is not).
    pub(crate) fn container_view(&self, view: u64) -> Result<(usize, AmmlScreenKind), AmmlStatus> {
        let address = self.enter_at(view, Self::CONTAINER_VERSION)?;
        let (layout, walk) = self.reads()?;
        let controller = self.book.controller_of_view(view).ok_or(AmmlStatus::REFUSED)?;
        if !ClientAdapter::call_bool(controller, Some(layout.container_slot))? {
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: `enter_at` proved `address` is the live view of a `ScreenRender` dispatch running on this thread.
        let kind = unsafe { walk.kind(address) }.unwrap_or(AmmlScreenKind::OTHER);
        Ok((controller, kind))
    }

    /// After the game handled a hover (`entry` is `ScreenDetours::HOVER`) or an unhover of the slot `slot` of the
    /// collection `name` on `controller`: raises the event with the screen id of `controller`, given now when the
    /// screen was not drawn yet. Runs on the thread of the game's handler, which is logged once.
    pub fn hovered(&self, entry: &'static str, controller: *mut c_void, name: *const c_void, slot: i32) {
        let unhover = entry == ScreenDetours::UNHOVER;
        let id = if unhover { ContainerSlotUnhover::ID } else { ContainerSlotHover::ID };
        let hub = self.state.events();
        if name.is_null() || !hub.has_listeners(id) {
            return;
        }
        // The game hovers the slot under a still pointer in the first frame of a screen, before its first draw, so
        // the hover can be the first event that names the controller.
        let screen = self.book.note_controller(controller.expose_provenance());
        let context = self.context(None);
        let note = if unhover { &self.unhover_thread } else { &self.hover_thread };
        if note.changed(context.thread) {
            curia::info!("ui thread", { hook: entry, game: context.thread == AmmlThread::GAME });
        }
        self.note_order(screen);
        // SAFETY: `name` is the `std::string const&` the game passed to its handler, live for the call.
        let text = unsafe { &*name.cast::<MsvcString>() }.to_str().unwrap_or("");
        let slot = AmmlSlotAt { collection: CollectionNames::collection_of(text), index: slot };
        if unhover {
            let mut event = ContainerSlotUnhover {
                header: EventHub::header::<ContainerSlotUnhover>(AmmlPhase::AFTER),
                screen,
                slot,
            };
            hub.raise(&mut event, &context);
        } else {
            let mut event =
                ContainerSlotHover { header: EventHub::header::<ContainerSlotHover>(AmmlPhase::AFTER), screen, slot };
            hub.raise(&mut event, &context);
        }
    }

    // The UI profile the last client tick read; `NOT_AVAILABLE` before the first tick and for a value other than 0
    // and 1. The option is read on the game thread only (`cache_profile`).
    fn profile(&self) -> Result<AmmlUiProfile, AmmlStatus> {
        match self.profile.load(Ordering::Relaxed) {
            0 => Ok(AmmlUiProfile::CLASSIC),
            1 => Ok(AmmlUiProfile::POCKET),
            _ => Err(AmmlStatus::NOT_AVAILABLE),
        }
    }

    // Reads the UI profile option through the option registry of the primary client and keeps it. Runs at the client
    // tick, on the game thread, where the client calls are allowed.
    fn cache_profile(&self) {
        let value = self.read_profile().map_or(Self::PROFILE_UNKNOWN, |profile| profile.0);
        self.profile.store(value, Ordering::Relaxed);
    }

    fn read_profile(&self) -> Result<AmmlUiProfile, AmmlStatus> {
        let (layout, _) = self.reads()?;
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let options = ClientAdapter::call_pointer(client, Some(layout.options_slot))?;
        let function =
            ClientAdapter::virtual_fn(options, Some(layout.ui_profile_slot)).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `UIProfile IOptionRegistry::getUIProfile() const`: `this` in rcx, the enum in eax.
        let function = unsafe { std::mem::transmute::<usize, ProfileFn>(function) };
        // SAFETY: called on its own object, the live option registry of the primary client.
        match unsafe { function(options) } {
            0 => Ok(AmmlUiProfile::CLASSIC),
            1 => Ok(AmmlUiProfile::POCKET),
            _ => Err(AmmlStatus::NOT_AVAILABLE),
        }
    }
}

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlEvent, AmmlInputMode, AmmlPhase, AmmlThread, ItemHovertext, UiPointer, UiScroll};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::PointerUpGate;

use crate::{
    EventHub, HovertextDetours, LoaderState, ObjectRead, ScreenAdapter, ThreadNote, UiInputDetours, UiInputLayout,
};

static ADAPTER: OnceLock<UiInputAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// The UI input events: `input.ui_pointer` (a pointer button of a screen view), `input.ui_scroll` (a wheel step of a
/// scene) and `ui.item_hovertext` (the hover text of a shulker box item), all raised Before the game acts.
///
/// A pointer event is raised for the two pointer button ids of the view only; any other id goes to the game with no
/// event. The view and screen ids come from the screen adapter's book. `PointerUpGate` keeps the up of a cancelled
/// down away from the game. A scroll event is raised for one wheel step of a scene, with the screen of the scene's
/// view when the loader can read it. The hover text event lets a listener hide the contents lines; the detour then
/// runs the base `Item` function in place of the shulker box override.
///
/// Each detour loads `UiFlags` once and returns to the game's code when no listener of its event exists. Each
/// adapter path logs its thread once (`ui thread`).
pub struct UiInputAdapter {
    state: &'static LoaderState,
    layout: Option<UiInputLayout>,
    base_hovertext: Option<usize>,
    gate: PointerUpGate,
    pointer_thread: ThreadNote,
    scroll_thread: ThreadNote,
    hovertext_thread: ThreadNote,
}

impl UiInputAdapter {
    pub const UI_POINTER: &'static str = UiPointer::ID;
    pub const UI_SCROLL: &'static str = UiScroll::ID;
    pub const ITEM_HOVERTEXT: &'static str = ItemHovertext::ID;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the button handler, the direction handler and the shulker box hover text for the rows whose catalog
    /// shape resolved, offers each event whose hook is in, and logs one line per row. Works once per process: a
    /// second call hooks nothing and gives the catalog rows. The screen adapter must be installed first: the
    /// pointer and scroll events take their ids from its book.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the ui input adapter is already installed");
            return rows;
        }
        let shaped = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let layout = UiInputLayout::new(state);
        let ids = ScreenAdapter::get().is_some() && layout.is_some();
        let pointer_on =
            ids && shaped(Self::UI_POINTER) && UiInputDetours::button_hook().install(state, Self::PRIORITY);
        let scroll_on =
            ids && shaped(Self::UI_SCROLL) && UiInputDetours::direction_hook().install(state, Self::PRIORITY);
        let base = state.resolve(HovertextDetours::BASE).ok().and_then(|address| usize::try_from(address).ok());
        let hovertext_on =
            base.is_some() && shaped(Self::ITEM_HOVERTEXT) && HovertextDetours::hook().install(state, Self::PRIORITY);
        if !Self::install_with(state, layout, base) {
            return rows;
        }
        for row in &mut rows {
            let on = match row.id {
                Self::UI_POINTER => pointer_on,
                Self::UI_SCROLL => scroll_on,
                _ => hovertext_on,
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
                _ => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// Installs the adapter over `layout` (without it the pointer and scroll events are off), with the address of the base hover text function. False when an adapter
    /// is already installed. Offers and hooks nothing.
    pub fn install_with(
        state: &'static LoaderState,
        layout: Option<UiInputLayout>,
        base_hovertext: Option<usize>,
    ) -> bool {
        let adapter = Self {
            state,
            layout,
            base_hovertext,
            gate: PointerUpGate::new(),
            pointer_thread: ThreadNote::new(),
            scroll_thread: ThreadNote::new(),
            hovertext_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the ui input adapter is already installed");
            return false;
        }
        true
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static UiInputAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&UiInputAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        [Self::UI_POINTER, Self::UI_SCROLL, Self::ITEM_HOVERTEXT].contains(&row.id)
    }

    // The context of an event raised on this thread, and a line in the log the first time a path runs on a thread.
    fn context(&self, note: &ThreadNote, hook: &'static str) -> amml_abi::AmmlEventContext {
        let context = ScreenAdapter::context_of(self.state, None);
        if note.changed(context.thread) {
            curia::info!("ui thread", { hook: hook, game: context.thread == AmmlThread::GAME });
        }
        context
    }

    /// Before the game handles button `button` of the view at `view`: raises `UiPointer` when `button` is one of the
    /// view's two pointer button ids. True when the game must not get the button event. Every view of the stack gets
    /// the button event, so this runs once for each.
    pub fn pointer(&self, view: usize, button: u32, pressed: bool) -> bool {
        let hub = self.state.events();
        if !hub.has_listeners(UiPointer::ID) {
            return false;
        }
        let Some(screens) = ScreenAdapter::get() else { return false };
        let Some(layout) = &self.layout else { return false };
        // SAFETY: `view` is the live `ScreenView` the game passed to its own handler; `mPointerButtonIds` is two u32.
        let Some(ids) = (unsafe { ObjectRead::value::<[u32; 2]>(view, layout.button_ids) }) else {
            return false;
        };
        let Some(pointer) = PointerUpGate::pointer_of(ids, button) else { return false };
        // SAFETY: as above; `mController` is a pointer member.
        let Some(controller) = (unsafe { ObjectRead::pointer(view, layout.controller) }) else { return false };
        let (screen, view_id) = screens.book().note(controller, view);
        // SAFETY: as above; the pointer position is two f32.
        let at = unsafe { ObjectRead::value::<[f32; 2]>(view, layout.pointer) }.unwrap_or([0.0; 2]);
        let at = if at.iter().all(|value| value.is_finite()) { at } else { [0.0; 2] };
        let held = !pressed && self.gate.up(view_id, pointer);
        let mut event = UiPointer {
            header: EventHub::header::<UiPointer>(AmmlPhase::BEFORE),
            screen,
            view: view_id,
            pointer,
            pressed: u8::from(pressed),
            input_mode: self.input_mode(view),
            _reserved: 0,
            x: at[0],
            y: at[1],
            _reserved2: 0,
        };
        // The up of a cancelled down starts cancelled: its listeners see it so, and the game does not get it.
        event.header.cancelled = u8::from(held);
        let context = self.context(&self.pointer_thread, UiInputDetours::BUTTON);
        let pristine = event;
        hub.raise_keeping(&mut event, &context, &|value| {
            let header = value.header;
            *value = pristine;
            value.header = header;
        });
        let cancelled = event.header.cancelled != 0;
        if pressed { !self.gate.down(view_id, pointer, cancelled) } else { held }
    }

    /// Forgets the hold-backs of cancelled downs, when the last listener of `UiPointer` is gone.
    pub fn forget_pointers(&self) {
        self.gate.clear();
    }

    // The input mode of the view as an `AmmlInputMode` value; 0 (unknown) without the member or for a value other
    // than the game's four.
    fn input_mode(&self, view: usize) -> u8 {
        let Some(offset) = self.layout.as_ref().and_then(|layout| layout.input_mode) else {
            return AmmlInputMode::UNKNOWN.0;
        };
        // SAFETY: `view` is the live `ScreenView` the game passed to its own handler; the input mode is a 32-bit enum.
        match unsafe { ObjectRead::value::<u32>(view, offset) } {
            Some(1) => AmmlInputMode::MOUSE.0,
            Some(2) => AmmlInputMode::TOUCH.0,
            Some(3) => AmmlInputMode::GAME_PAD.0,
            Some(4) => AmmlInputMode::MOTION_CONTROLLER.0,
            _ => AmmlInputMode::UNKNOWN.0,
        }
    }

    /// Before the scene at `scene` handles one wheel step `y` (the game's vertical component): raises `UiScroll`.
    /// True when a listener cancelled it, so the scene must not get the step. A step of 0 or one that is not finite
    /// is no wheel notch and raises nothing. `notches` is the sign of `y`: positive for a step away from the user.
    pub fn scroll(&self, scene: usize, y: f32) -> bool {
        let hub = self.state.events();
        if !hub.has_listeners(UiScroll::ID) || !y.is_finite() || y == 0.0 {
            return false;
        }
        let mut event = UiScroll {
            header: EventHub::header::<UiScroll>(AmmlPhase::BEFORE),
            screen: self.screen_of_scene(scene),
            notches: if y > 0.0 { 1 } else { -1 },
            _reserved: 0,
        };
        let context = self.context(&self.scroll_thread, UiInputDetours::DIRECTION);
        let pristine = event;
        hub.raise_keeping(&mut event, &context, &|value| {
            let header = value.header;
            *value = pristine;
            value.header = header;
        });
        event.header.cancelled != 0
    }

    // The screen id of the controller of the view of `scene`; 0 when a link of that chain cannot be read.
    fn screen_of_scene(&self, scene: usize) -> u64 {
        let Some(screens) = ScreenAdapter::get() else { return 0 };
        let Some(layout) = &self.layout else { return 0 };
        // SAFETY: `scene` is the live `UIScene` the game passed to its own handler; `mScreenView` is a pointer member.
        let Some(view) = (unsafe { ObjectRead::pointer(scene, layout.scene_view) }) else { return 0 };
        // SAFETY: the pointer in `mScreenView` is the live `ScreenView` of the scene; `mController` is a pointer member.
        let Some(controller) = (unsafe { ObjectRead::pointer(view, layout.controller) }) else { return 0 };
        screens.book().note(controller, view).0
    }

    /// Before the game builds the hover text of a shulker box item: raises `ItemHovertext`. Gives the address of the
    /// base `Item` function when a listener left `contents_lines` at 0 (the detour then calls it in place of the
    /// game's override), else `None`.
    pub fn hidden_base(&self) -> Option<usize> {
        let hub = self.state.events();
        if !hub.has_listeners(ItemHovertext::ID) {
            return None;
        }
        let mut event = ItemHovertext {
            header: EventHub::header::<ItemHovertext>(AmmlPhase::BEFORE),
            kind: ItemHovertext::CONTAINER_ITEM,
            contents_lines: 1,
            _reserved: [0; 3],
        };
        let context = self.context(&self.hovertext_thread, HovertextDetours::HOVERTEXT);
        hub.raise_keeping(&mut event, &context, &|value| {
            value.kind = ItemHovertext::CONTAINER_ITEM;
            value._reserved = [0; 3];
        });
        if event.contents_lines == 0 { self.base_hovertext } else { None }
    }
}

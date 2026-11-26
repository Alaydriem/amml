use std::ffi::c_void;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use amml_abi::{AmmlClientFlag, AmmlEvent, AmmlPhase, AmmlReleaseReason, AmmlScreen, AmmlStatus, MouseGrab};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::{FunctionStorage, MemberCall, MsvcHeap, MsvcString};
use amml_runtime::RestoreFlag;

use crate::{
    ClientDetours, EventHub, GameGate, LeaveHold, LoaderState, NonOwnerPointerLayout, PauseOption, ScreenNames,
    SessionAdapter, SessionHook, WorldRenderAdapter,
};

static ADAPTER: OnceLock<ClientAdapter> = OnceLock::new();

/// `LocalPlayer* getLocalPlayer() const`, `IOptionRegistry& getOptions()`, `IMinecraftGame& getMinecraftGame()`:
/// rcx the object, the pointer in rax.
type PointerFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
/// `bool isWorldActive() const`, `bool isShowingMenu() const`, `bool getHideHud() const`: rcx the object,
/// the bool in al.
type BoolFn = unsafe extern "C" fn(*mut c_void) -> u8;
/// `void IOptionRegistry::setHideGUI(bool)`: rcx the option registry, the bool in dl.
type SetBoolFn = unsafe extern "C" fn(*mut c_void, u8);
/// `_Do_call` of a `std::function<void(IClientInstance&)>` callable: rcx the callable, rdx the client.
type AnyInputCallFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// The client areas (`client.read`, `client.top_screen`, `client.hud`, `client.mouse`, `client.pause`) and
/// the `client.mouse_grab` event, over the primary `ClientInstance` of the session adapter.
///
/// Every call needs the game gate open, the game thread, and the handle of the current primary client
/// (`NOT_READY`, then `REFUSED`). Client calls are not bound to a world: they work on the title screen too,
/// and `local_player` gives null while no world is joined. A handle stays valid while it is the primary
/// client, so the same `ClientInstance` after a later join is accepted again.
///
/// Mouse grab state is what the loader last saw: the game's own grab and release (through the detours) and
/// the loader's own writes, which call past the detours and raise no `MouseGrab`. The values from before a
/// mod's first set are kept in the loader's `RestoreBook`; what the last mod that set a flag leaves is
/// written back at the next `ClientTick` of the primary client. The mouse grab is game state, not a player
/// setting, so it is written back only while it still holds the value that mod set; when the game grabbed or
/// released the mouse since, it is left as the game has it.
pub struct ClientAdapter {
    state: &'static LoaderState,
    // Slots of the IClientInstance vtable.
    local_player: Option<usize>,
    world_active: Option<usize>,
    showing_menu: Option<usize>,
    top_screen_name: Option<usize>,
    options: Option<usize>,
    minecraft_game: Option<usize>,
    // The slot of the MinecraftGame vtable that holds `isInGame`.
    is_in_game: Option<usize>,
    // Slots of the IOptionRegistry vtable.
    get_hide_hud: Option<usize>,
    set_hide_gui: Option<usize>,
    get_if_valid: Option<usize>,
    is_graphics_deferred: Option<usize>,
    // The `ClientInstance.getMinecraftInput` slot and the derived offsets of the any-input chain.
    minecraft_input: Option<usize>,
    input_handler_at: Option<usize>,
    any_input_at: Option<usize>,
    // The last `note_activity` result logged plus one (0 before the first), and the lines written.
    activity_logged: AtomicU8,
    activity_lines: AtomicU8,
    pause_option: PauseOption,
    // The `MinecraftGame.vtable` global, the `MinecraftGame.AppPlatformListener.vtable` global and the
    // offset of the AppPlatformListener subobject in MinecraftGame.
    game_vtable: Option<usize>,
    listener_vtable: Option<usize>,
    listener_offset: Option<usize>,
    captured: AtomicU8,
    // The `MOUSE_CAPTURED` value each mod (by owner index) last set.
    mouse_values: Mutex<Vec<(usize, bool)>>,
    // Values to write back: the flag, the value, and for the mouse the value the released mod set, which
    // the grab must still hold for the write to run.
    pending: Mutex<Vec<(RestoreFlag, bool, Option<bool>)>>,
    // The top screen names already logged, so a game build with new screen names shows them once each.
    screen_names_seen: Mutex<Vec<String>>,
    leave_hold: LeaveHold,
}

impl ClientAdapter {
    pub const READ: &'static str = "client.read";
    pub const TOP_SCREEN: &'static str = "client.top_screen";
    pub const HUD: &'static str = "client.hud";
    pub const MOUSE: &'static str = "client.mouse";
    pub const PAUSE: &'static str = "client.pause";
    pub const ACTIVITY: &'static str = "client.activity";
    pub const IN_GAMEPLAY: &'static str = "client.in_gameplay";
    const AREA_VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    const CAPTURE_UNKNOWN: u8 = 0;
    // The most distinct top screen names the adapter logs.
    const MAX_SCREEN_NAMES: usize = 32;
    const CAPTURE_FREE: u8 = 1;
    const CAPTURE_HELD: u8 = 2;
    // The most `note_activity` result lines the adapter logs.
    const MAX_ACTIVITY_LINES: u8 = 8;

    /// Hooks the mouse and focus functions whose entries resolved, offers each client area and the
    /// `client.mouse_grab` event whose catalog shape resolved and whose hooks were installed, and logs one
    /// `area adapter` / `area not available` (or `event offered` / `event not available`) line per row.
    /// Returns those rows. Works once per process: a second call hooks nothing and gives the catalog rows.
    ///
    /// The focus hook is installed whenever `MinecraftGame.onAppFocusLost` resolves: a focus loss releases
    /// the injected input also on a build without the pause area.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_client_row).collect();
        let slot = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            local_player: slot("ClientInstance.getLocalPlayer"),
            world_active: slot("ClientInstance.isWorldActive"),
            showing_menu: slot("ClientInstance.isShowingMenu"),
            top_screen_name: slot("ClientInstance.getTopScreenName"),
            options: slot("ClientInstance.getOptions"),
            minecraft_game: slot("ClientInstance.getMinecraftGame"),
            is_in_game: slot("MinecraftGame.isInGame"),
            get_hide_hud: slot("IOptionRegistry.getHideHud"),
            set_hide_gui: slot("IOptionRegistry.setHideGUI"),
            get_if_valid: slot("OptionRegistry.getIfValid"),
            is_graphics_deferred: slot("IOptionRegistry.isGraphicsModeDeferred"),
            minecraft_input: slot("ClientInstance.getMinecraftInput"),
            input_handler_at: slot("MinecraftInputHandler.mInputHandler"),
            any_input_at: slot("InputHandler.mAnyInputHandler"),
            activity_logged: AtomicU8::new(0),
            activity_lines: AtomicU8::new(0),
            pause_option: PauseOption::new(),
            game_vtable: slot(AdapterCatalog::MINECRAFT_GAME_VTABLE),
            listener_vtable: slot("MinecraftGame.AppPlatformListener.vtable"),
            listener_offset: slot("MinecraftGame.appPlatformListenerOffset"),
            captured: AtomicU8::new(Self::CAPTURE_UNKNOWN),
            mouse_values: Mutex::new(Vec::new()),
            pending: Mutex::new(Vec::new()),
            screen_names_seen: Mutex::new(Vec::new()),
            leave_hold: LeaveHold::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the client adapter is already installed");
            return rows;
        }
        let hooks = ClientDetours::hooks();
        let install =
            |entry: &str| hooks.iter().find(|hook| hook.entry == entry).is_some_and(|hook| Self::hook(state, hook));
        let focus_hooked = resolved(ClientDetours::FOCUS_LOST) && install(ClientDetours::FOCUS_LOST);
        let mouse_shape = rows.iter().any(|row| row.id == Self::MOUSE && row.shape.is_some());
        let mut mouse_refused: Vec<&'static str> = Vec::new();
        if mouse_shape {
            for entry in [ClientDetours::GRAB, ClientDetours::RELEASE] {
                if !install(entry) {
                    mouse_refused.push(entry);
                }
            }
        }
        for row in &mut rows {
            let refused: Vec<&'static str> = match row.id {
                Self::MOUSE | MouseGrab::ID => mouse_refused.clone(),
                Self::PAUSE if !focus_hooked => vec![ClientDetours::FOCUS_LOST],
                _ => Vec::new(),
            };
            if row.shape.is_some() && !refused.is_empty() {
                row.shape = None;
                row.missing = refused;
            }
            Self::offer(state, row);
        }
        rows
    }

    // The rows this adapter offers: the client areas and `client.mouse_grab`.
    fn is_client_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Area => {
                [Self::READ, Self::TOP_SCREEN, Self::HUD, Self::MOUSE, Self::PAUSE, Self::ACTIVITY, Self::IN_GAMEPLAY]
                    .contains(&row.id)
            }
            NeedKind::Event => row.id == MouseGrab::ID,
        }
    }

    fn hook(state: &'static LoaderState, hook: &SessionHook) -> bool {
        hook.install(state, Self::PRIORITY)
    }

    // Offers `row` when it has a shape and logs the outcome.
    fn offer(state: &LoaderState, row: &Availability) {
        match (row.kind, row.shape) {
            (NeedKind::Area, Some(shape)) => {
                state.areas().offer(row.id, row.version);
                curia::info!("area adapter", { area: row.id, shape: shape });
            }
            (NeedKind::Area, None) => {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") })
            }
            (NeedKind::Event, Some(shape)) => {
                state.events().offer(row.id, row.version);
                curia::info!("event offered", { event: row.id, shape: shape });
            }
            (NeedKind::Event, None) => {
                curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
            }
        }
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static ClientAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ClientAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn gate(&self) -> &GameGate {
        self.state.events().gate()
    }

    /// The address of `client` when a client call may use it: the game gate is open (`NOT_READY`), the
    /// caller is on the game thread and `client` is the current primary client (`REFUSED`).
    pub fn check(&self, client: *const c_void) -> Result<usize, AmmlStatus> {
        self.gate().require_open()?;
        if !self.gate().on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let client = client.expose_provenance();
        match SessionAdapter::primary_client() {
            Some(primary) if primary == client => Ok(client),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // `check` behind the game gate and the offer of `area` (`NOT_AVAILABLE`).
    fn enter(&self, client: *const c_void, area: &str) -> Result<usize, AmmlStatus> {
        self.gate().require_open()?;
        if !self.state.areas().is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.check(client)
    }

    /// Whether the game says a world is being played (`MinecraftGame::isInGame` on the game of the last frame), asked
    /// for the primary `client`. `NOT_AVAILABLE` without area `client.in_gameplay`; `NOT_READY` before the first frame.
    pub fn in_gameplay(&self, client: *const c_void) -> Result<bool, AmmlStatus> {
        self.enter(client, Self::IN_GAMEPLAY)?;
        let game = SessionAdapter::minecraft_game().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Self::call_bool(game, self.is_in_game)
    }

    /// The local player of `client`, 0 for none; 0 with no game call while no world is joined.
    pub fn local_player(&self, client: *const c_void) -> Result<usize, AmmlStatus> {
        let client = self.enter(client, Self::READ)?;
        if self.gate().epoch().0 == 0 {
            return Ok(0);
        }
        Self::call_pointer(client, self.local_player)
    }

    /// The local player of the primary client, for the loader's own input path: `None` when the gate is
    /// closed, the caller is not on the game thread, no world is joined, there is no primary client or the
    /// read gives null. Needs no client area: only the `getLocalPlayer` slot.
    pub(crate) fn primary_local_player(&self) -> Option<usize> {
        if self.gate().epoch().0 == 0 {
            return None;
        }
        self.primary_local_player_any_epoch()
    }

    /// The local player of the primary client, also before the loader's world join (after the game's `StartGame` the
    /// player exists before the first client update that joins): `None` when the gate is closed, off the game thread,
    /// without a primary client or a player.
    pub(crate) fn primary_local_player_any_epoch(&self) -> Option<usize> {
        self.gate().require_open().ok()?;
        if !self.gate().on_game_thread() {
            return None;
        }
        let client = SessionAdapter::primary_client()?;
        Self::call_pointer(client, self.local_player).ok().filter(|player| *player != 0)
    }

    /// One client flag. `MOUSE_CAPTURED` is `NOT_AVAILABLE` until the loader saw the first grab or release;
    /// an unknown flag is `INVALID_ARGUMENT`.
    pub fn flag(&self, client: *const c_void, flag: AmmlClientFlag) -> Result<bool, AmmlStatus> {
        match flag {
            AmmlClientFlag::WORLD_ACTIVE => Self::call_bool(self.enter(client, Self::READ)?, self.world_active),
            AmmlClientFlag::SHOWING_MENU => Self::call_bool(self.enter(client, Self::READ)?, self.showing_menu),
            AmmlClientFlag::MOUSE_CAPTURED => {
                self.enter(client, Self::MOUSE)?;
                self.captured().ok_or(AmmlStatus::NOT_AVAILABLE)
            }
            AmmlClientFlag::HIDE_HUD => {
                let options = self.options_of(self.enter(client, Self::HUD)?)?;
                Self::call_bool(options, self.get_hide_hud)
            }
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// A world leave starts: the title is held back until the leave is complete (`LeaveHold`).
    pub(crate) fn leave_started(&self) {
        self.leave_hold.begin(Instant::now());
    }

    /// The screen on top of `client`. After a world leave, a title read is `LOADING` until the leave is complete
    /// (`LeaveHold`). The game's string is freed on the game heap; `NOT_AVAILABLE` when the heap is not available,
    /// so the string would leak.
    pub fn top_screen(&self, client: *const c_void) -> Result<AmmlScreen, AmmlStatus> {
        self.top_screen_named(client).map(|(screen, _)| screen)
    }

    /// `top_screen` and the game's top screen name it was read from (empty when the name is not UTF-8).
    pub(crate) fn top_screen_named(&self, client: *const c_void) -> Result<(AmmlScreen, String), AmmlStatus> {
        let client = self.enter(client, Self::TOP_SCREEN)?;
        if !self.state.heap().is_available() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let function = Self::virtual_fn(client, self.top_screen_name).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let this = std::ptr::with_exposed_provenance_mut::<c_void>(client);
        // SAFETY: the slot holds `std::string IClientInstance::getTopScreenName() const`, a member function that
        // returns the string through a hidden pointer; `MsvcString` has the layout of the release std::string.
        let mut name = unsafe { MemberCall::call_sret0::<MsvcString>(function as *const c_void, this) };
        let text = std::str::from_utf8(name.as_bytes()).ok();
        let in_world = self.gate().epoch().0 != 0;
        let screen = text.map_or(AmmlScreen::OTHER, |text| ScreenNames::map(text, in_world));
        let screen = self.leave_hold.settle(screen, Instant::now());
        if let Some(text) = text {
            self.note_screen_name(text, in_world);
        }
        let owned = text.unwrap_or("").to_string();
        let heap = MsvcHeap::new(Self::heap_new, Self::heap_delete);
        // SAFETY: the game built the string with its own allocator, which `heap` calls; it is not used again.
        unsafe { name.release(&heap) };
        Ok((screen, owned))
    }

    // Logs `top screen name` the first time a name is read, for up to `MAX_SCREEN_NAMES` names.
    fn note_screen_name(&self, name: &str, in_world: bool) {
        let mut seen = self.screen_names_seen.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if seen.len() >= Self::MAX_SCREEN_NAMES || seen.iter().any(|known| known == name) {
            return;
        }
        seen.push(name.to_string());
        curia::info!("top screen name", { name: name, screen: format!("{:?}", ScreenNames::map(name, in_world)) });
    }

    /// Sets `MOUSE_CAPTURED` or `HIDE_HUD` for the mod at `owner`; any other flag is `INVALID_ARGUMENT`.
    /// The mouse write calls `grabMouse` / `releaseMouse` past the loader's detours, so no `MouseGrab` is
    /// raised for it. The value before the first set of any mod is kept for restore.
    pub fn set_flag(
        &self,
        owner: usize,
        client: *const c_void,
        flag: AmmlClientFlag,
        value: bool,
    ) -> Result<(), AmmlStatus> {
        match flag {
            AmmlClientFlag::MOUSE_CAPTURED => {
                let game = self.game_of(self.enter(client, Self::MOUSE)?)?;
                // The hooks are in place before the game runs, so no grab seen means the mouse is free.
                let before = self.captured().unwrap_or(false);
                self.write_captured(game, value)?;
                self.state.restores().set(owner, RestoreFlag::MouseCaptured, before);
                let mut values = Self::recover(self.mouse_values.lock());
                values.retain(|(held, _)| *held != owner);
                values.push((owner, value));
                Ok(())
            }
            AmmlClientFlag::HIDE_HUD => {
                let options = self.options_of(self.enter(client, Self::HUD)?)?;
                let before = Self::call_bool(options, self.get_hide_hud)?;
                Self::call_set_bool(options, self.set_hide_gui, value)?;
                self.state.restores().set(owner, RestoreFlag::HideHud, before);
                Ok(())
            }
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// Opens the pause screen of `client` by calling `MinecraftGame::onAppFocusLost` on the
    /// AppPlatformListener subobject, past the loader's detour, then releases every injected input with
    /// `FOCUS_LOST`, as the detour does after a real focus loss.
    /// The subobject is at the derived `MinecraftGame.appPlatformListenerOffset`; when the qword there is
    /// not the `MinecraftGame.AppPlatformListener.vtable` address, nothing is called and the result is
    /// `NOT_AVAILABLE`, with a log line.
    ///
    /// The game opens the pause screen there only when its option `pause_menu_on_focus_lost` is on, so the
    /// option is read first: off gives `REFUSED` with an info line and nothing is called; an option that
    /// cannot be read (see `PauseOption`) gives `NOT_AVAILABLE`.
    pub fn open_pause_screen(&self, client: *const c_void) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::PAUSE)?;
        let listener = self.listener_of(client)?;
        let options = self.options_of(client)?;
        let get_if_valid = Self::virtual_fn(options, self.get_if_valid).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `options` is the live option registry of the primary client, on the game thread, and
        // `get_if_valid` is the function in its `OptionRegistry.getIfValid` slot.
        let enabled = unsafe { self.pause_option.read(options, get_if_valid) }?;
        if !enabled {
            curia::info!("pause screen refused: the game option is off", { option: PauseOption::SAVE_TAG });
            return Err(AmmlStatus::REFUSED);
        }
        self.run_focus_lost(listener)
    }

    /// The AppPlatformListener subobject of the MinecraftGame of `client`, at the derived
    /// `MinecraftGame.appPlatformListenerOffset`. When the qword there is not the
    /// `MinecraftGame.AppPlatformListener.vtable` address, the result is `NOT_AVAILABLE`, with a log line.
    pub(crate) fn listener_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        let game = self.game_of(client)?;
        let (Some(offset), Some(expected)) = (self.listener_offset, self.listener_vtable) else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        };
        let listener = game.checked_add(offset).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `game` is the live MinecraftGame of the primary client and `offset` is the derived offset of
        // a subobject inside it, so the qword at `listener` is readable; it is 8-aligned like every vtable
        // pointer of the object.
        let found = unsafe { std::ptr::with_exposed_provenance::<usize>(listener).read() };
        if found != expected {
            curia::warn!("the AppPlatformListener vtable is not at the derived offset", {
                offset: format!("{offset:#x}"),
                found: format!("{found:#x}"),
                expected: format!("{expected:#x}")
            });
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(listener)
    }

    /// Calls the game's `onAppFocusLost` on `listener` past the loader's detour, then releases every injected input
    /// with `FOCUS_LOST`, as the detour does after a real focus loss. `NOT_AVAILABLE` when the hook is not installed.
    pub(crate) fn run_focus_lost(&self, listener: usize) -> Result<(), AmmlStatus> {
        // SAFETY: `listener` is the AppPlatformListener subobject of a live MinecraftGame (its vtable checked by
        // `listener_of`).
        let called = unsafe { ClientDetours::focus_lost_past_detour(std::ptr::with_exposed_provenance_mut(listener)) };
        if !called {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.focus_lost();
        Ok(())
    }

    /// Runs the game's any-input handler of `client` once: `getMinecraftInput()->mInputHandler->mAnyInputHandler`
    /// called with the client, the code the game runs for each input event. No input event is raised. `REFUSED`
    /// when the input handler is not valid (its flag byte is not 1) or null, its `InputHandler` is null or no
    /// handler is set; nothing is called then. The one use of the control block that `getMinecraftInput` hands out
    /// is given back in every case. The first result, and each result that differs from the one before it, is
    /// logged (`note_activity`), at most `MAX_ACTIVITY_LINES` times.
    pub fn note_activity(&self, client: *const c_void) -> Result<(), AmmlStatus> {
        let result = self.run_any_input(client);
        self.note_activity_result(result);
        result
    }

    fn run_any_input(&self, client: *const c_void) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::ACTIVITY)?;
        let (Some(handler_at), Some(any_input_at)) = (self.input_handler_at, self.any_input_at) else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        };
        let function = Self::virtual_fn(client, self.minecraft_input).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let this = std::ptr::with_exposed_provenance_mut::<c_void>(client);
        // SAFETY: the slot holds `Bedrock::NonOwnerPointer<MinecraftInputHandler> getMinecraftInput() const`, a
        // member function that writes the 24-byte pointer through a hidden result pointer.
        let mut input = unsafe { MemberCall::call_sret0::<NonOwnerPointerLayout>(function as *const c_void, this) };
        // SAFETY: `input` is the pointer the game just handed out, `client` the live primary client on the game
        // thread, and the offsets are derived for this build.
        let result = unsafe { Self::call_any_input(&input, client, handler_at, any_input_at) };
        // SAFETY: the copy the game handed out holds one use of the control block (or none when it is empty).
        unsafe { input.control.release() };
        result
    }

    // Walks `mInputHandler` and `mAnyInputHandler` from the handler in `input` and calls the handler's `_Do_call`
    // with `client`.
    //
    // Safety: `input` was written by `getMinecraftInput` of the live client `client`, on the game thread;
    // `handler_at` and `any_input_at` are the derived offsets of this build.
    unsafe fn call_any_input(
        input: &NonOwnerPointerLayout,
        client: usize,
        handler_at: usize,
        any_input_at: usize,
    ) -> Result<(), AmmlStatus> {
        if input.control.get().copied() != Some(1) || input.pointer.is_null() {
            return Err(AmmlStatus::REFUSED);
        }
        let handler = input.pointer.expose_provenance().checked_add(handler_at).ok_or(AmmlStatus::REFUSED)?;
        // SAFETY: a valid MinecraftInputHandler holds its `mInputHandler` unique_ptr at the derived offset.
        let input_handler = unsafe { std::ptr::with_exposed_provenance::<usize>(handler).read_unaligned() };
        if input_handler == 0 {
            return Err(AmmlStatus::REFUSED);
        }
        let storage = input_handler.checked_add(any_input_at).ok_or(AmmlStatus::REFUSED)?;
        // SAFETY: a live InputHandler holds the `mAnyInputHandler` std::function at the derived offset.
        let callable =
            unsafe { (&raw const (*std::ptr::with_exposed_provenance::<FunctionStorage>(storage)).callable).read() };
        let do_call = Self::virtual_fn(callable.expose_provenance(), Some(FunctionStorage::DO_CALL_SLOT))
            .ok_or(AmmlStatus::REFUSED)?;
        // SAFETY: slot 2 of a non-empty std::function<void(IClientInstance&)> callable is `_Do_call(callable, client)`.
        let do_call = unsafe { std::mem::transmute::<usize, AnyInputCallFn>(do_call) };
        // SAFETY: the callable and the client are live; the game's input dispatcher calls it the same way.
        unsafe { do_call(callable, std::ptr::with_exposed_provenance_mut(client)) };
        Ok(())
    }

    fn note_activity_result(&self, result: Result<(), AmmlStatus>) {
        let (code, name) = match result {
            Ok(()) => (1, "ok"),
            Err(AmmlStatus::REFUSED) => (2, "refused"),
            Err(AmmlStatus::NOT_AVAILABLE) => (3, "not_available"),
            Err(AmmlStatus::NOT_READY) => (4, "not_ready"),
            Err(_) => (5, "failed"),
        };
        if self.activity_logged.swap(code, Ordering::AcqRel) == code {
            return;
        }
        let logged = self
            .activity_lines
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < Self::MAX_ACTIVITY_LINES).then_some(n + 1));
        if logged.is_ok() {
            curia::info!("note_activity", { result: name });
        }
    }

    /// Queues values to write back for the released mod at `owner`, from `RestoreBook::release_owner`. They
    /// are written at the next `ClientTick` of the primary client; `MOUSE_CAPTURED` only when the grab then
    /// still holds the value `owner` set. With the game gate closed (before the first frame or after
    /// shutdown), or no client adapter, they are dropped with a `restore dropped` log line.
    pub fn queue_restore(owner: usize, values: Vec<(RestoreFlag, bool)>) {
        let set_by_owner = ADAPTER.get().and_then(|adapter| adapter.forget_mouse_value(owner));
        if values.is_empty() {
            return;
        }
        let Some(adapter) = ADAPTER.get() else {
            curia::info!("restore dropped", { count: values.len(), reason: "no client adapter" });
            return;
        };
        if !adapter.gate().is_open() {
            curia::info!("restore dropped", { count: values.len(), reason: "the game does not run" });
            return;
        }
        let values = values.into_iter().map(|(flag, value)| {
            let expected = if flag == RestoreFlag::MouseCaptured { set_by_owner } else { None };
            (flag, value, expected)
        });
        adapter.lock_pending().extend(values);
    }

    // Forgets the `MOUSE_CAPTURED` value the mod at `owner` set and gives it.
    fn forget_mouse_value(&self, owner: usize) -> Option<bool> {
        let mut values = Self::recover(self.mouse_values.lock());
        let index = values.iter().position(|(held, _)| *held == owner)?;
        Some(values.remove(index).1)
    }

    /// Writes back the queued values on `client` now, with the same checks as `apply_restores` (the mouse
    /// grab only while it still holds the value the released mod set). The live shutdown calls it on the
    /// game thread after the last disable and before the gate closes: no `ClientTick` follows.
    pub(crate) fn flush_restores(&self, client: usize) {
        self.apply_restores(client);
    }

    /// Writes back the queued values on the primary client `client`. Called by the session adapter before
    /// each `ClientTick`, on the game thread.
    pub(crate) fn apply_restores(&self, client: usize) {
        let values = std::mem::take(&mut *self.lock_pending());
        if values.is_empty() {
            return;
        }
        if !self.gate().is_open() {
            curia::info!("restore dropped", { count: values.len(), reason: "the game does not run" });
            return;
        }
        for (flag, value, expected) in values {
            if let Some(expected) = expected
                && self.captured() != Some(expected)
            {
                curia::info!("client flag left", {
                    flag: format!("{flag:?}"),
                    reason: "the game changed it after the mod set it"
                });
                continue;
            }
            let written = match flag {
                RestoreFlag::MouseCaptured => self.game_of(client).and_then(|game| self.write_captured(game, value)),
                RestoreFlag::HideHud => {
                    self.options_of(client).and_then(|options| Self::call_set_bool(options, self.set_hide_gui, value))
                }
            };
            match written {
                Ok(()) => curia::info!("client flag restored", { flag: format!("{flag:?}"), value: value }),
                Err(status) => {
                    curia::warn!("a client flag was not restored", { flag: format!("{flag:?}"), status: status.0 });
                }
            }
        }
    }

    /// Before the game's grab: raises `MouseGrab` with the primary client as the context client. False when
    /// a listener cancelled it, so the game does not grab.
    pub(crate) fn grab_requested(&self) -> bool {
        let hub = self.state.events();
        let client = SessionAdapter::primary_client().unwrap_or(0);
        let context = amml_abi::AmmlEventContext {
            client: std::ptr::with_exposed_provenance::<c_void>(client),
            ..hub.loader_context()
        };
        let mut event = MouseGrab { header: EventHub::header::<MouseGrab>(AmmlPhase::BEFORE) };
        hub.raise(&mut event, &context);
        event.header.cancelled == 0
    }

    /// At a `ClientTick` of the primary client: reads the game's graphics mode (`isGraphicsModeDeferred`, true in
    /// Vibrant Visuals) and gives it to the world render, which raises it as `WorldRendered.vibrant` and frees the
    /// meshes when it changes. Reads nothing while the world render is not installed, no option slot resolved, or no
    /// mod listens to `WorldRendered` or holds a mesh.
    pub(crate) fn note_graphics_mode(&self, client: usize) {
        if !WorldRenderAdapter::with(false, WorldRenderAdapter::reads_graphics_mode) {
            return;
        }
        let Ok(options) = self.options_of(client) else { return };
        if let Ok(deferred) = Self::call_bool(options, self.is_graphics_deferred) {
            WorldRenderAdapter::with((), |adapter| adapter.set_vibrant(deferred));
        }
    }

    /// Records the grab state the game or the loader just set.
    pub(crate) fn note_captured(&self, captured: bool) {
        let value = if captured { Self::CAPTURE_HELD } else { Self::CAPTURE_FREE };
        self.captured.store(value, Ordering::Release);
    }

    /// After the game's `onAppFocusLost`: every injected input is released.
    pub(crate) fn focus_lost(&self) {
        self.state.input().release_everything(AmmlReleaseReason::FOCUS_LOST);
    }

    fn captured(&self) -> Option<bool> {
        match self.captured.load(Ordering::Acquire) {
            Self::CAPTURE_HELD => Some(true),
            Self::CAPTURE_FREE => Some(false),
            _ => None,
        }
    }

    // Grabs or releases the mouse of `game` past the loader's detours and records the new state.
    fn write_captured(&self, game: usize, captured: bool) -> Result<(), AmmlStatus> {
        let this = std::ptr::with_exposed_provenance_mut::<c_void>(game);
        // SAFETY: `game_of` checked that `game` is a live MinecraftGame with the `MinecraftGame.vtable` vtable.
        let called = unsafe {
            if captured { ClientDetours::grab_past_detour(this) } else { ClientDetours::release_past_detour(this) }
        };
        if !called {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.note_captured(captured);
        Ok(())
    }

    // The MinecraftGame of `client`. When `MinecraftGame.vtable` resolved, its vtable must be that global;
    // a null game or another vtable gives `NOT_AVAILABLE`, with a log line.
    pub(crate) fn game_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        let game = Self::call_pointer(client, self.minecraft_game)?;
        if game == 0 {
            curia::warn!("a client call found no MinecraftGame");
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if let Some(expected) = self.game_vtable {
            // SAFETY: `game` is the non-null MinecraftGame the client gave; its first qword is its vtable pointer.
            let found = unsafe { std::ptr::with_exposed_provenance::<usize>(game).read() };
            if found != expected {
                curia::warn!("a client call found a MinecraftGame with another vtable", {
                    found: format!("{found:#x}"),
                    expected: format!("{expected:#x}")
                });
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
        }
        Ok(game)
    }

    // The option registry of `client`; null gives `NOT_AVAILABLE`.
    fn options_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        match Self::call_pointer(client, self.options)? {
            0 => Err(AmmlStatus::NOT_AVAILABLE),
            options => Ok(options),
        }
    }

    // The function in `slot` of the vtable at the start of `object`; `None` for an unknown slot, or a null
    // object, vtable or function.
    pub(crate) fn virtual_fn(object: usize, slot: Option<usize>) -> Option<usize> {
        let slot = slot?;
        if object == 0 {
            return None;
        }
        // SAFETY: `object` is a live, non-null game object (the primary client, its option registry or its
        // MinecraftGame, all used on the game thread); its first qword is its vtable pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<*const usize>(object).read() };
        if vtable.is_null() {
            return None;
        }
        // SAFETY: `vtable` is the non-null vtable of `object`; the slot entry's recipe range check keeps `slot`
        // inside it.
        let function = unsafe { vtable.add(slot).read() };
        (function != 0).then_some(function)
    }

    pub(crate) fn call_pointer(object: usize, slot: Option<usize>) -> Result<usize, AmmlStatus> {
        let function = Self::virtual_fn(object, slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds a member function that takes only `this` and returns a pointer.
        let function = unsafe { std::mem::transmute::<usize, PointerFn>(function) };
        // SAFETY: called on its own object.
        Ok(unsafe { function(std::ptr::with_exposed_provenance_mut(object)) }.expose_provenance())
    }

    pub(crate) fn call_bool(object: usize, slot: Option<usize>) -> Result<bool, AmmlStatus> {
        let function = Self::virtual_fn(object, slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds a const member function that takes only `this` and returns a bool.
        let function = unsafe { std::mem::transmute::<usize, BoolFn>(function) };
        // SAFETY: called on its own object.
        Ok(unsafe { function(std::ptr::with_exposed_provenance_mut(object)) } != 0)
    }

    fn call_set_bool(object: usize, slot: Option<usize>, value: bool) -> Result<(), AmmlStatus> {
        let function = Self::virtual_fn(object, slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `void setHideGUI(bool)`.
        let function = unsafe { std::mem::transmute::<usize, SetBoolFn>(function) };
        // SAFETY: called on its own object with a bool.
        unsafe { function(std::ptr::with_exposed_provenance_mut(object), u8::from(value)) };
        Ok(())
    }

    fn lock_pending(&self) -> MutexGuard<'_, Vec<(RestoreFlag, bool, Option<bool>)>> {
        Self::recover(self.pending.lock())
    }

    // The queued restores and mouse values are plain data, so a lock poisoned by a panic is recovered.
    fn recover<T>(locked: std::sync::LockResult<MutexGuard<'_, T>>) -> MutexGuard<'_, T> {
        locked.unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) extern "C" fn heap_new(size: usize) -> *mut u8 {
        LoaderState::get().map_or(std::ptr::null_mut(), |state| state.heap().alloc(size))
    }

    pub(crate) extern "C" fn heap_delete(ptr: *mut u8, size: usize) {
        if let Some(state) = LoaderState::get() {
            // SAFETY: `MsvcHeap` passes back a block of the game allocator, with its size.
            unsafe { state.heap().free(ptr, size) };
        }
    }
}

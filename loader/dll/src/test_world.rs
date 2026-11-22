use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use amml_abi::{AmmlScreen, AmmlStatus};
use amml_data::AdapterCatalog;
use amml_msvc::{MsvcHeap, MsvcString};

use crate::{ClientAdapter, LoaderState, SessionAdapter, StorageDetours};

static TEST_WORLD: OnceLock<TestWorld> = OnceLock::new();

/// `World::WorldSystem& getWorldSystem() const`: rcx the MinecraftGame, the pointer in rax.
type WorldSystemFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
/// `StartLocalWorldResult LocalWorldStarter::startLocalWorld(const World::WorldID&)`: rcx the starter, rdx
/// the WorldID (one MSVC `std::string`), the result in eax.
type StartLocalWorldFn = unsafe extern "C" fn(*mut c_void, *const MsvcString) -> u32;

/// The loader's test hook that starts one local world from the start screen, so an in-game test run reaches
/// a world without a hand on the keyboard. The level id (the world folder name) comes from the
/// `AMML_TEST_WORLD` variable, read by the standalone host only; it is not offered to mods.
///
/// `frame` runs after each `FrameUpdate`, on the game thread. It acts only while a level id is set, it has not
/// stopped and no world is joined. It waits until the title screen has settled: the top screen of the primary client
/// was the start screen and no world storage was live in every frame of the last `settle` (15 s for the test world
/// from the variable, so the game's Microsoft sign-in completes first); any other frame, a toast screen for example,
/// starts the wait again. Then it calls `LocalWorldStarter.startLocalWorld` with a WorldID built on the game heap and
/// freed after the call. Results 2 (startup already in progress) and 3 (game server starter not ready) retry 60 frames
/// later, at most 30 times; any other result but 0, a missing entry or a null hop stops it with a `test world not
/// started` line. A joined world stops it.
///
/// The world is started at most once per process: after result 0 it watches the load, and when the top screen was the
/// loading screen and is then the start screen again with no world joined (the game refused the load, for example with
/// disconnect reason 46 when its Microsoft sign-in was not complete), it stops with a `test world load ended without a
/// join` line. A second start in the same process crashes the game.
pub struct TestWorld {
    level_id: Option<String>,
    stopped: AtomicBool,
    frames: AtomicU32,
    next_attempt: AtomicU32,
    retries: AtomicU32,
    started: AtomicBool,
    loading_seen: AtomicBool,
    settle: Duration,
    quiet_since: Mutex<Option<Instant>>,
}

impl TestWorld {
    pub const VARIABLE: &'static str = "AMML_TEST_WORLD";
    pub const GET_WORLD_SYSTEM: &'static str = "MinecraftGame.getWorldSystem";
    pub const LOCAL_WORLD_STARTER: &'static str = "WorldSystem.mLocalWorldStarter";
    pub const START_LOCAL_WORLD: &'static str = "LocalWorldStarter.startLocalWorld";
    /// The entries `frame` uses besides the client adapter's.
    pub const ENTRIES: [&'static str; 3] = [Self::GET_WORLD_SYSTEM, Self::LOCAL_WORLD_STARTER, Self::START_LOCAL_WORLD];
    const STARTED: u32 = 0;
    const STARTUP_ALREADY_IN_PROGRESS: u32 = 2;
    const GAME_SERVER_STARTER_NOT_READY: u32 = 3;
    const RETRY_FRAMES: u32 = 60;
    const MAX_RETRIES: u32 = 30;
    /// How long the title screen stays settled before the start, for the test world from the variable.
    pub const START_SETTLE: Duration = Duration::from_secs(15);

    /// A test world for `level_id`; `None` (or an empty id) does nothing. The start waits until the title screen has
    /// been settled for `settle`.
    pub fn new(level_id: Option<String>, settle: Duration) -> Self {
        Self {
            level_id: level_id.filter(|id| !id.is_empty()),
            stopped: AtomicBool::new(false),
            frames: AtomicU32::new(0),
            next_attempt: AtomicU32::new(0),
            retries: AtomicU32::new(0),
            started: AtomicBool::new(false),
            loading_seen: AtomicBool::new(false),
            settle,
            quiet_since: Mutex::new(None),
        }
    }

    /// A test world for the level id in `AMML_TEST_WORLD`; unset, empty or not Unicode gives none.
    pub fn from_env() -> Self {
        Self::new(std::env::var(Self::VARIABLE).ok(), Self::START_SETTLE)
    }

    pub fn level_id(&self) -> Option<&str> {
        self.level_id.as_deref()
    }

    /// Makes this the test world that the session adapter runs after each `FrameUpdate`. Works once per
    /// process: a second call keeps the first.
    pub fn install(self) {
        if TEST_WORLD.set(self).is_err() {
            curia::warn!("the test world is already installed");
        }
    }

    /// Runs `body` on the installed test world; nothing before `install`.
    pub(crate) fn with(body: impl FnOnce(&TestWorld)) {
        if let Some(world) = TEST_WORLD.get() {
            body(world);
        }
    }

    /// One frame of the test world, on the game thread, after the `FrameUpdate` listeners.
    pub fn frame(&self) {
        let Some(level_id) = self.level_id.as_deref() else {
            return;
        };
        if self.stopped.load(Ordering::Acquire) {
            return;
        }
        let Some(state) = LoaderState::get() else {
            return;
        };
        if state.events().gate().epoch().0 != 0 {
            curia::info!("test world stopped: a world is joined", { level_id: level_id });
            self.stop();
            return;
        }
        let frame = self.frames.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if self.started.load(Ordering::Acquire) {
            return self.watch(level_id);
        }
        if frame < self.next_attempt.load(Ordering::Acquire) {
            return;
        }
        match self.settled(level_id) {
            Ok(true) => {}
            Ok(false) => return,
            Err(missing) => return self.not_started_missing(level_id, missing),
        }
        let result = match self.start(state, level_id) {
            Ok(result) => result,
            Err(missing) => return self.not_started_missing(level_id, missing),
        };
        match result {
            Self::STARTED => {
                curia::info!("test world started", { level_id: level_id });
                self.loading_seen.store(false, Ordering::Release);
                self.started.store(true, Ordering::Release);
            }
            Self::STARTUP_ALREADY_IN_PROGRESS | Self::GAME_SERVER_STARTER_NOT_READY
                if self.retries.load(Ordering::Acquire) < Self::MAX_RETRIES =>
            {
                let retry = self.retries.fetch_add(1, Ordering::AcqRel).saturating_add(1);
                self.next_attempt.store(frame.saturating_add(Self::RETRY_FRAMES), Ordering::Release);
                curia::info!("test world not ready, retrying", { level_id: level_id, result: result, retry: retry });
            }
            _ => {
                curia::warn!("test world not started", { level_id: level_id, result: result });
                self.stop();
            }
        }
    }

    fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }

    // One frame after the start: notes the loading screen; the start screen after it, with no world joined, stops the
    // test world.
    fn watch(&self, level_id: &str) {
        let screen = match self.top_screen() {
            Ok(Some(screen)) => screen,
            Ok(None) => return,
            Err(missing) => return self.not_started_missing(level_id, missing),
        };
        if screen == AmmlScreen::LOADING {
            self.loading_seen.store(true, Ordering::Release);
            return;
        }
        if screen != AmmlScreen::START || !self.loading_seen.load(Ordering::Acquire) {
            return;
        }
        curia::warn!("test world load ended without a join", { level_id: level_id });
        self.stop();
    }

    // Whether the title screen has been settled for `settle`: the start screen on top and no live world storage in every
    // frame since the first such frame. Any other frame clears that first frame.
    fn settled(&self, level_id: &str) -> Result<bool, &'static str> {
        let on_start = self.top_screen()? == Some(AmmlScreen::START);
        let storage_live = StorageDetours::with_live(|_| ()).is_some();
        let mut quiet = self.quiet();
        if !on_start || storage_live {
            *quiet = None;
            return Ok(false);
        }
        let now = Instant::now();
        let since = *quiet.get_or_insert(now);
        let waited = now.saturating_duration_since(since);
        if waited < self.settle {
            return Ok(false);
        }
        let waited_ms = u64::try_from(waited.as_millis()).unwrap_or(u64::MAX);
        let settle_ms = u64::try_from(self.settle.as_millis()).unwrap_or(u64::MAX);
        curia::info!("test world settled", { level_id: level_id, settle_ms: settle_ms, waited_ms: waited_ms });
        Ok(true)
    }

    fn quiet(&self) -> std::sync::MutexGuard<'_, Option<Instant>> {
        self.quiet_since.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn not_started_missing(&self, level_id: &str, missing: &str) {
        curia::warn!("test world not started", { level_id: level_id, missing: missing });
        self.stop();
    }

    // The top screen of the primary client. A gate that is not open yet, another thread or no primary client gives
    // `None` (wait); no client adapter or no top-screen area is missing.
    fn top_screen(&self) -> Result<Option<AmmlScreen>, &'static str> {
        let client = SessionAdapter::primary_client().unwrap_or(0);
        let adapter = ClientAdapter::get().ok_or(ClientAdapter::TOP_SCREEN)?;
        match adapter.top_screen(std::ptr::with_exposed_provenance(client)) {
            Ok(screen) => Ok(Some(screen)),
            Err(AmmlStatus::NOT_READY | AmmlStatus::REFUSED) => Ok(None),
            Err(_) => Err(ClientAdapter::TOP_SCREEN),
        }
    }

    // Calls `startLocalWorld` on the LocalWorldStarter of the frame's MinecraftGame with `level_id` and gives
    // its result; the name of the missing entry or the null hop otherwise.
    fn start(&self, state: &LoaderState, level_id: &str) -> Result<u32, &'static str> {
        let slot = Self::entry(state, Self::GET_WORLD_SYSTEM)?;
        let field = Self::entry(state, Self::LOCAL_WORLD_STARTER)?;
        let function = Self::entry(state, Self::START_LOCAL_WORLD)?;
        if !state.heap().is_available() {
            return Err("core.operator_new, core.operator_delete");
        }
        let game = SessionAdapter::minecraft_game().ok_or("MinecraftGame")?;
        if let Ok(expected) = Self::entry(state, AdapterCatalog::MINECRAFT_GAME_VTABLE) {
            // SAFETY: `game` is the non-null MinecraftGame of the running frame; its first qword is its vtable.
            let found = unsafe { std::ptr::with_exposed_provenance::<usize>(game).read() };
            if found != expected {
                return Err(AdapterCatalog::MINECRAFT_GAME_VTABLE);
            }
        }
        // SAFETY: `game` is a live MinecraftGame (its vtable checked when the global resolved); its first qword
        // is its vtable pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<*const usize>(game).read() };
        if vtable.is_null() {
            return Err(AdapterCatalog::MINECRAFT_GAME_VTABLE);
        }
        // SAFETY: `vtable` is the MinecraftGame vtable, which holds `getWorldSystem` at the derived `slot`.
        let get_world_system = unsafe { vtable.add(slot).read() };
        if get_world_system == 0 {
            return Err(Self::GET_WORLD_SYSTEM);
        }
        // SAFETY: the slot holds `World::WorldSystem& getWorldSystem() const`.
        let get_world_system = unsafe { std::mem::transmute::<usize, WorldSystemFn>(get_world_system) };
        // SAFETY: called on its own MinecraftGame, on the game thread.
        let world_system = unsafe { get_world_system(std::ptr::with_exposed_provenance_mut(game)) }.expose_provenance();
        if world_system == 0 {
            return Err(Self::GET_WORLD_SYSTEM);
        }
        let member = world_system.checked_add(field).ok_or(Self::LOCAL_WORLD_STARTER)?;
        // SAFETY: `member` is the derived `mLocalWorldStarter` member inside the live WorldSystem, an 8-byte
        // aligned `std::unique_ptr`.
        let starter = unsafe { std::ptr::with_exposed_provenance::<usize>(member).read() };
        if starter == 0 {
            return Err(Self::LOCAL_WORLD_STARTER);
        }
        let heap = MsvcHeap::new(Self::heap_new, Self::heap_delete);
        // SAFETY: `heap` calls the game's `operator new` and `operator delete`, which the loader resolved.
        let mut world_id = unsafe { MsvcString::new_in(&heap, level_id) }.map_err(|_| "core.operator_new")?;
        // SAFETY: the entry is the function `startLocalWorld` of the running game module.
        let start_local_world = unsafe { std::mem::transmute::<usize, StartLocalWorldFn>(function) };
        // SAFETY: `starter` is the live LocalWorldStarter of the WorldSystem and `world_id` a valid MSVC string
        // that outlives the call; the game reads the id through the const reference only.
        let result = unsafe { start_local_world(std::ptr::with_exposed_provenance_mut(starter), &raw const world_id) };
        // SAFETY: the string's block, if any, came from `heap` and the game kept no reference to it.
        unsafe { world_id.release(&heap) };
        Ok(result)
    }

    // The resolved value of `id` as a `usize`; the id when it did not resolve.
    fn entry(state: &LoaderState, id: &'static str) -> Result<usize, &'static str> {
        state.resolve(id).ok().and_then(|value| usize::try_from(value).ok()).ok_or(id)
    }

    extern "C" fn heap_new(size: usize) -> *mut u8 {
        LoaderState::get().map_or(std::ptr::null_mut(), |state| state.heap().alloc(size))
    }

    extern "C" fn heap_delete(ptr: *mut u8, size: usize) {
        if let Some(state) = LoaderState::get() {
            // SAFETY: `MsvcHeap` passes back a block of the game allocator, with its size.
            unsafe { state.heap().free(ptr, size) };
        }
    }
}

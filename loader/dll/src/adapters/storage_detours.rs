use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_abi::AmmlThread;

use crate::{Boundary, JoinEvents, LoaderState, SessionHook, StorageReadGuard, ThreadNote};

/// `DBStorage::DBStorage(DBStorageConfig, NotNullNonOwnerPtr<LevelDbEnv>)`: rcx this, rdx a pointer to the config, r8 a
/// pointer to the env pointer; gives this.
type CtorFn = unsafe extern "C" fn(usize, usize, usize) -> usize;
/// `bool DBStorage::checkShutdownDone()`: rcx this; the result is the low byte of the return register, and the whole
/// register is passed on.
type ShutdownFn = unsafe extern "C" fn(usize) -> u64;

static CTOR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SHUTDOWN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// The address of the `DBStorage` made last and not yet shut down; 0 while there is none.
static LIVE: AtomicUsize = AtomicUsize::new(0);
// The readers inside `with_live`.
static READERS: AtomicUsize = AtomicUsize::new(0);
static LONG_WAIT_LOGGED: AtomicBool = AtomicBool::new(false);
// The entries of the hooks the first `install` refused; set once, by that call.
static REFUSED: OnceLock<Vec<&'static str>> = OnceLock::new();
// The offset of `mFullPath` in a `DBStorage`; 0 until `install` resolved it.
static PATH_AT: AtomicUsize = AtomicUsize::new(0);
// The storages that were opened and have not reported their shutdown done. Locked only when a storage is opened and
// when `checkShutdownDone` gives done.
static OPEN: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static CTOR_THREAD: ThreadNote = ThreadNote::new();
static SHUTDOWN_THREAD: ThreadNote = ThreadNote::new();

/// The loader's detours of the `DBStorage` constructor and `checkShutdownDone`, which keep the address of the live
/// storage of the world the game opened.
///
/// The constructor runs on a storage thread and publishes its `this` once the game's constructor returned (the folder
/// path is set by then). `checkShutdownDone` withdraws it on entry when the call is on the live storage, and for every storage waits
/// until no `with_live` reader is inside before it calls the game's function; the game destroys the storage only after
/// that call, so a reader never sees a destroyed storage. A call on another storage with no reader inside is two loads and
/// the original. Neither detour locks, allocates or logs on that path.
///
/// The constructor detour also raises `WorldStorageOpened` (through `JoinEvents`) and `checkShutdownDone` raises
/// `WorldStorageClosed` the first time it gives done for a storage that was opened; the lock of the open list is taken
/// only for those two.
pub struct StorageDetours;

impl StorageDetours {
    pub const CTOR: &'static str = "DBStorage.ctor";
    pub const SHUTDOWN_DONE: &'static str = "DBStorage.checkShutdownDone";
    // The spins a shutdown makes for a reader before it yields the thread; a reader copies one path.
    const SPINS: u32 = 100_000;

    pub fn hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::CTOR,
                vtable: None,
                detour: Self::ctor as CtorFn as *const c_void,
                original: &CTOR,
            },
            SessionHook {
                entry: Self::SHUTDOWN_DONE,
                vtable: None,
                detour: Self::shutdown_done as ShutdownFn as *const c_void,
                original: &SHUTDOWN,
            },
        ]
    }

    /// Installs both detours as loader hooks at `priority` and gives the entries that were refused (none when both
    /// stand). The hooks are installed once per process, by the first call; a later call gives the first result, so
    /// the adapters that need the storage hooks (`WorldInfoAdapter`, `JoinEvents`) can each call it.
    pub fn install(state: &'static LoaderState, priority: i32) -> Vec<&'static str> {
        REFUSED
            .get_or_init(|| {
                let path_at = state.resolve("DBStorage.mFullPath").ok().and_then(|at| usize::try_from(at).ok());
                PATH_AT.store(path_at.unwrap_or(0), Ordering::Release);
                Self::hooks().iter().filter(|hook| !hook.install(state, priority)).map(|hook| hook.entry).collect()
            })
            .clone()
    }

    /// Runs `read` with the address of the live storage while it cannot be destroyed; `None` when there is none.
    pub fn with_live<R>(read: impl FnOnce(usize) -> R) -> Option<R> {
        let _guard = StorageReadGuard::new(&READERS);
        let storage = LIVE.load(Ordering::SeqCst);
        (storage != 0).then(|| read(storage))
    }

    /// Publishes `storage` as the live one, as the constructor detour does.
    pub fn set_live(storage: usize) {
        LIVE.store(storage, Ordering::SeqCst);
    }

    /// Withdraws `storage` when it is the live one, then waits until no reader is inside `with_live`, whatever `storage`
    /// is: a reader may hold the address of a storage that is no longer the live one. True when `storage` was live.
    /// The wait has no bound: it spins, then yields, and logs one warning when it is long.
    pub fn withdraw(storage: usize) -> bool {
        let was_live = storage != 0
            && LIVE.load(Ordering::SeqCst) == storage
            && LIVE.compare_exchange(storage, 0, Ordering::SeqCst, Ordering::SeqCst).is_ok();
        let mut spins = 0u32;
        while READERS.load(Ordering::SeqCst) != 0 {
            spins = spins.saturating_add(1);
            if spins < Self::SPINS {
                std::hint::spin_loop();
                continue;
            }
            if spins == Self::SPINS {
                Self::warn_long_wait();
            }
            std::thread::yield_now();
        }
        was_live
    }

    fn warn_long_wait() {
        if !LONG_WAIT_LOGGED.swap(true, Ordering::Relaxed) {
            curia::warn!("a storage shutdown waits for a reader of the world folder");
        }
    }

    fn note(note: &ThreadNote, entry: &'static str) {
        let on_game_thread = LoaderState::get().is_some_and(|state| state.events().gate().on_game_thread());
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if note.changed(thread) {
            curia::info!("world thread", { hook: entry, game: on_game_thread });
        }
    }

    unsafe extern "C" fn ctor(this: usize, config: usize, env: usize) -> usize {
        // SAFETY: the hook on `DBStorage.ctor` is installed, so its slot holds a `CtorFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, CtorFn>(CTOR.load(Ordering::Acquire)) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, config, env) };
        if this != 0 {
            Self::set_live(this);
            Boundary::run((), || Self::note(&CTOR_THREAD, Self::CTOR));
            Boundary::run((), || Self::opened(this));
        }
        result
    }

    unsafe extern "C" fn shutdown_done(this: usize) -> u64 {
        // SAFETY: the hook on `DBStorage.checkShutdownDone` is installed, so its slot holds a `ShutdownFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, ShutdownFn>(SHUTDOWN.load(Ordering::Acquire)) };
        if Self::withdraw(this) {
            Boundary::run((), || Self::note(&SHUTDOWN_THREAD, Self::SHUTDOWN_DONE));
        }
        // SAFETY: the game's own argument, passed on unchanged.
        let result = unsafe { original(this) };
        if result & 0xFF != 0 && this != 0 {
            Boundary::run((), || Self::closed(this));
        }
        result
    }

    // Records `storage` as opened and raises `WorldStorageOpened` for it.
    fn opened(storage: usize) {
        let mut open = OPEN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !open.contains(&storage) {
            open.push(storage);
        }
        drop(open);
        let path_at = PATH_AT.load(Ordering::Acquire);
        if path_at != 0 {
            JoinEvents::with(|events| events.storage_opened(storage, path_at));
        }
    }

    // Raises `WorldStorageClosed` once for a storage that was opened and not yet reported done.
    fn closed(storage: usize) {
        let mut open = OPEN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(index) = open.iter().position(|address| *address == storage) else {
            return;
        };
        open.swap_remove(index);
        drop(open);
        JoinEvents::with(JoinEvents::storage_closed);
    }
}

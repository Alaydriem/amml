use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_abi::AmmlPhase;
use amml_data::AdapterCatalog;

use crate::{Boundary, JoinNames, SessionAdapter, SessionHook};

/// `void IMinecraftGame::update()`: rcx the MinecraftGame.
type GameUpdateFn = unsafe extern "C" fn(*mut c_void);
/// `bool ClientInstance::update(bool isInitFinished, bool)`: rcx the ClientInstance, rdx and r8 passed on as
/// whole registers; the bool in al.
type ClientUpdateFn = unsafe extern "C" fn(*mut c_void, u64, u64) -> u8;
/// `void ClientInstance::onTick(int nTick, int maxTick)`: rcx the GameCallbacks subobject, edx, r8d.
type OnTickFn = unsafe extern "C" fn(*mut c_void, i32, i32);
/// `void IClientInstance::onStartJoinGame(...)`: rcx the ClientInstance, rdx the local flag in its low byte, r8 and r9
/// two `std::string const&`, and the stack arguments at entry+28h (a third `std::string const&`), +30h, +38h and +40h, each passed on as the whole
/// 8-byte register or stack slot.
type StartJoinFn = unsafe extern "C" fn(*mut c_void, u64, u64, u64, u64, u64, u64, u64);
/// `void ClientInstance::onLevelExit()`: rcx the GameCallbacks subobject.
type ExitLevelFn = unsafe extern "C" fn(*mut c_void);

static GAME_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CLIENT_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ON_TICK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static START_JOIN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static EXIT_LEVEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the session functions of the game. Each one calls the original exactly once,
/// with the arguments it received, and raises its events inside `Boundary::run`, so a panic while raising
/// never skips the original.
pub struct SessionDetours;

impl SessionDetours {
    pub const GAME_UPDATE: &'static str = "MinecraftGame.update";
    pub const CLIENT_UPDATE: &'static str = "ClientInstance.update";
    pub const ON_TICK: &'static str = "ClientInstance.onTick";
    pub const START_JOIN: &'static str = "ClientInstance.onStartJoinGame";
    pub const EXIT_LEVEL: &'static str = "ClientExitLevel.hook";

    /// The five hooks, in install order.
    pub fn hooks() -> [SessionHook; 5] {
        [
            SessionHook {
                entry: Self::GAME_UPDATE,
                vtable: Some(AdapterCatalog::MINECRAFT_GAME_VTABLE),
                detour: Self::game_update as GameUpdateFn as *const c_void,
                original: &GAME_UPDATE,
            },
            SessionHook {
                entry: Self::CLIENT_UPDATE,
                vtable: None,
                detour: Self::client_update as ClientUpdateFn as *const c_void,
                original: &CLIENT_UPDATE,
            },
            SessionHook {
                entry: Self::ON_TICK,
                vtable: None,
                detour: Self::on_tick as OnTickFn as *const c_void,
                original: &ON_TICK,
            },
            SessionHook {
                entry: Self::START_JOIN,
                vtable: Some(AdapterCatalog::CLIENT_INSTANCE_VTABLE),
                detour: Self::start_join as StartJoinFn as *const c_void,
                original: &START_JOIN,
            },
            SessionHook {
                entry: Self::EXIT_LEVEL,
                vtable: None,
                detour: Self::exit_level as ExitLevelFn as *const c_void,
                original: &EXIT_LEVEL,
            },
        ]
    }

    /// The original behind `slot`, as a function of type `F`.
    ///
    /// # Safety
    /// `slot` is the original slot of a hook whose target has the signature `F`, and the hook is installed.
    unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot to the next hook or the trampoline before it moved the thunk to
        // this detour; both have the signature `F`, which the caller guarantees is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    unsafe extern "C" fn game_update(this: *mut c_void) {
        Boundary::run((), || SessionAdapter::with(|adapter| adapter.frame(AmmlPhase::BEFORE, this)));
        // SAFETY: the hook on `MinecraftGame.update` is installed, so its slot holds a `GameUpdateFn`.
        let original = unsafe { Self::original::<GameUpdateFn>(&GAME_UPDATE) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
        Boundary::run((), || SessionAdapter::with(|adapter| adapter.frame(AmmlPhase::AFTER, this)));
    }

    unsafe extern "C" fn client_update(this: *mut c_void, init_finished: u64, extra: u64) -> u8 {
        // SAFETY: the hook on `ClientInstance.update` is installed, so its slot holds a `ClientUpdateFn`.
        let original = unsafe { Self::original::<ClientUpdateFn>(&CLIENT_UPDATE) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, init_finished, extra) };
        Boundary::run((), || SessionAdapter::with(|adapter| adapter.client_updated(this)));
        result
    }

    unsafe extern "C" fn on_tick(this: *mut c_void, tick: i32, max_tick: i32) {
        // SAFETY: the hook on `ClientInstance.onTick` is installed, so its slot holds an `OnTickFn`.
        let original = unsafe { Self::original::<OnTickFn>(&ON_TICK) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, tick, max_tick) };
        Boundary::run((), || SessionAdapter::with(|adapter| adapter.ticked(this, tick)));
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn start_join(this: *mut c_void, a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64) {
        // SAFETY: the hook on `ClientInstance.onStartJoinGame` is installed, so its slot holds a `StartJoinFn`.
        let original = unsafe { Self::original::<StartJoinFn>(&START_JOIN) };
        // SAFETY: rdx holds the local flag in its low byte, and r8, r9 and the stack word at entry+28h are the
        // addresses of the three live `std::string const&` arguments of the call, which the caller keeps alive until the call returns.
        let names = Boundary::run(JoinNames::default(), || unsafe { JoinNames::read(a, b, c, d) });
        // SAFETY: the game's own registers and stack slots, passed on unchanged.
        unsafe { original(this, a, b, c, d, e, f, g) };
        Boundary::run((), || SessionAdapter::with(|adapter| adapter.join_started(&names)));
    }

    unsafe extern "C" fn exit_level(this: *mut c_void) {
        Boundary::run((), || SessionAdapter::with(SessionAdapter::leaving));
        // SAFETY: the hook on `ClientExitLevel.hook` is installed, so its slot holds an `ExitLevelFn`.
        let original = unsafe { Self::original::<ExitLevelFn>(&EXIT_LEVEL) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
    }
}

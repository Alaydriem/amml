use std::cell::Cell;
use std::ffi::c_void;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};
use std::thread::LocalKey;

use amml_abi::{AmmlBlockPos, AmmlFace, AmmlHand, BuildBlock};
use amml_runtime::BuildReentry;

use crate::{Boundary, PlayerActionAdapter, PlayerActionDetours, SessionHook};

/// `static bool ClientInputCallbacks::handleBuildAction(IClientInstance&, BuildActionIntention&, HitResult const& solid,
/// HitResult const& liquid)`: rcx the client, rdx the intention, r8 and r9 the two hit results; the bool in al.
type HandleBuildActionFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_void, *const c_void) -> u8;
/// `bool GameMode::buildBlock(BlockPos const&, uchar face, HandSlot, bool isSimTick)`: rcx the game mode, rdx the
/// position, r8b the face, r9b the hand slot, the isSimTick byte in the fifth argument slot; the bool in al. The detour
/// passes the whole registers and the whole stack slot on.
type BuildBlockFn = unsafe extern "C" fn(*mut c_void, *const AmmlBlockPos, u64, u64, u64) -> u8;

static USE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static BUILD_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the two detours have work for besides the game's own call. Zero: each detour calls the original and nothing
// else.
static WANTED: AtomicU32 = AtomicU32::new(0);
// The isSimTick argument of the last build of the local player on the game thread, in the low byte, and the world
// epoch of that build above it.
static LAST_SIM_TICK: AtomicU64 = AtomicU64::new(0);
static REENTRY: LazyLock<BuildReentry> = LazyLock::new(BuildReentry::new);

thread_local! {
    // Set while this thread decides one use press or one build, so a call from inside a listener raises no second
    // event of the same kind.
    static USING: Cell<bool> = const { Cell::new(false) };
    static BUILDING: Cell<bool> = const { Cell::new(false) };
}

/// The loader's detours behind `UsePressed` (on `ClientInputCallbacks::handleBuildAction`) and `BuildBlock` (on the
/// base body in slot 6 of the GameMode table, which the survival leaf wrapper of slot 6 jumps to), the flags word they
/// read, the isSimTick of the local player's last build, and the action scopes of the mods' own builds.
///
/// With no flag set, each detour calls the original and does nothing else: one relaxed load, no lock, no game read.
/// `USE_LISTENER` is set while a mod listens to `UsePressed`, `BUILD_LISTENER` while a mod listens to `BuildBlock`, and
/// `SIM_TICK` while area `player.build` is offered (the build detour then keeps the isSimTick of the local player's
/// builds on the game thread for `build_at`). The decision runs inside `Boundary::run`, so a panic counts as "not
/// cancelled" and the game's call runs once with its own arguments.
pub struct UseBuildDetours;

impl UseBuildDetours {
    pub const USE_ENTRY: &'static str = "ClientInputCallbacks.handleBuildAction";
    pub const BUILD_ENTRY: &'static str = "GameMode.buildBlock";
    pub const GAME_MODE_VTABLE: &'static str = "GameMode.vtable";
    /// A mod listens to `UsePressed`.
    pub const USE_LISTENER: u32 = 1;
    /// A mod listens to `BuildBlock`.
    pub const BUILD_LISTENER: u32 = 2;
    /// Area `player.build` is offered: keep the isSimTick of the local player's builds.
    pub const SIM_TICK: u32 = 4;
    /// What the game gets for a cancelled use press: "handled", as the old mod returned. Six of the seven callers on
    /// 26.63 and 70.26 test the result (`test al,al; je`) and only on true tail-call one client virtual with an action
    /// code (0Ah from five callers, 9 from the one at 26.63 0x6583e72 / 70.26 0x242980b; 26.63 `[rax+9B0h]`, body
    /// 0x65bcb60; 70.26 `[rax+8E0h]`, body 0x8b449a0), which stores the code in a new 4-byte block at client+2B0h /
    /// +2C0h. The seventh (26.63 0x65a3623 / 70.26 0x8b2cc50) does not test the result. No caller builds with either
    /// value.
    pub const HANDLED: u8 = 1;

    /// The hook on `handleBuildAction`.
    pub fn use_pressed() -> SessionHook {
        SessionHook {
            entry: Self::USE_ENTRY,
            vtable: None,
            detour: Self::handle_build_action as HandleBuildActionFn as *const c_void,
            original: &USE_ORIGINAL,
        }
    }

    /// The hook on the function in slot 6 of the GameMode table.
    pub fn build_block() -> SessionHook {
        SessionHook {
            entry: Self::BUILD_ENTRY,
            vtable: Some(Self::GAME_MODE_VTABLE),
            detour: Self::build_block_detour as BuildBlockFn as *const c_void,
            original: &BUILD_ORIGINAL,
        }
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// The flags word: one relaxed load.
    pub fn flags() -> u32 {
        WANTED.load(Ordering::Relaxed)
    }

    /// The isSimTick argument of the local player's last build on the game thread in the world of epoch `epoch`; 0
    /// before the first build of that world, so a value does not carry over a world change.
    pub fn last_sim_tick(epoch: u64) -> u8 {
        let packed = LAST_SIM_TICK.load(Ordering::Relaxed);
        if packed >> 8 == epoch { (packed & 0xFF) as u8 } else { 0 }
    }

    pub(crate) fn note_sim_tick(epoch: u64, sim_tick: u8) {
        LAST_SIM_TICK.store((epoch << 8) | u64::from(sim_tick), Ordering::Relaxed);
    }

    /// The action scopes of the mods' own builds, by thread.
    pub fn reentry() -> &'static BuildReentry {
        &REENTRY
    }

    // Runs one decision unless this thread is already deciding one of the kind `guard`; false (call the original) on
    // re-entry, on panic and before the player action adapter is installed.
    fn decide(guard: &'static LocalKey<Cell<bool>>, body: impl FnOnce(&PlayerActionAdapter) -> bool) -> bool {
        if guard.with(Cell::get) {
            return false;
        }
        guard.with(|flag| flag.set(true));
        let cancelled = Boundary::run(false, || PlayerActionAdapter::with(false, body));
        guard.with(|flag| flag.set(false));
        cancelled
    }

    unsafe extern "C" fn handle_build_action(
        client: *mut c_void,
        intention: *mut c_void,
        solid: *const c_void,
        liquid: *const c_void,
    ) -> u8 {
        // SAFETY: the hook on `ClientInputCallbacks.handleBuildAction` is installed, so its slot holds a
        // `HandleBuildActionFn`.
        let original = unsafe { PlayerActionDetours::original::<HandleBuildActionFn>(&USE_ORIGINAL) };
        if Self::flags() & Self::USE_LISTENER != 0 && Self::decide(&USING, PlayerActionAdapter::use_pressed) {
            return Self::HANDLED;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(client, intention, solid, liquid) }
    }

    unsafe extern "C" fn build_block_detour(
        this: *mut c_void,
        pos: *const AmmlBlockPos,
        face: u64,
        hand: u64,
        sim_tick: u64,
    ) -> u8 {
        // SAFETY: the hook on the `GameMode.buildBlock` slot is installed, so its slot holds a `BuildBlockFn`.
        let original = unsafe { PlayerActionDetours::original::<BuildBlockFn>(&BUILD_ORIGINAL) };
        let flags = Self::flags();
        if flags != 0 {
            let event = BuildBlock {
                header: PlayerActionAdapter::header::<BuildBlock>(),
                pos: PlayerActionDetours::position(pos),
                face: AmmlFace(face as u8),
                hand: AmmlHand(hand as u8),
                sim_tick: sim_tick as u8,
                is_local_player: 0,
            };
            if Self::decide(&BUILDING, |adapter| adapter.build_block(this, event, flags)) {
                return 0;
            }
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, pos, face, hand, sim_tick) }
    }
}

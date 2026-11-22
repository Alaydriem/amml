use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_abi::{AmmlBlockPos, BlockBreakContinue, BlockBreakStart, ItemUse, ItemUseOnBlock, PlayerAttack};

use crate::{Boundary, PlayerActionAdapter, SessionHook};

/// `bool GameMode::startDestroyBlock(BlockPos const&, uchar face, bool& hasDestroyedBlock)`: rcx the game mode, rdx
/// the position, r8 the face (passed on as the whole register), r9 the bool&; the bool in al.
type StartFn = unsafe extern "C" fn(*mut c_void, *const AmmlBlockPos, u64, *mut u8) -> u8;
/// `bool GameMode::continueDestroyBlock(BlockPos const&, uchar face, Vec3 const& playerPos, bool& hasDestroyedBlock)`:
/// as `StartFn` with the Vec3 in r9 and the bool& on the stack.
type ContinueFn = unsafe extern "C" fn(*mut c_void, *const AmmlBlockPos, u64, *const c_void, *mut u8) -> u8;
/// `bool GameMode::useItem(ItemStack&, bool)` (`GameMode.useItem`): rcx the game mode, rdx the stack, r8 the bool.
type UseItemBoolFn = unsafe extern "C" fn(*mut c_void, *mut c_void, u64) -> u8;
/// `InteractionResult GameMode::useItem(ItemStack&, bool)` (`GameMode.useItemInteraction`): rcx the game mode, rdx
/// the hidden result, r8 the stack, r9 the bool; rax the result pointer.
type UseItemInteractionFn = unsafe extern "C" fn(*mut c_void, *mut u8, *mut c_void, u64) -> *mut u8;
/// `bool GameMode::attack(Actor&, Vec3 const&)`: rcx the game mode, rdx the target, r8 the click position.
type AttackFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_void) -> u8;
/// `InteractionResult Item::useOn(ItemStack&, Actor&, BlockPos const&, uchar face, uchar, Vec3 const&, ...) const`:
/// rcx the item, rdx the hidden result, r8 the stack, r9 the actor, then the position, the face, a byte, the click
/// position and one more argument on the stack; rax the result pointer.
type UseOnFn = unsafe extern "C" fn(
    *mut c_void,
    *mut u8,
    *mut c_void,
    *mut c_void,
    *const AmmlBlockPos,
    u64,
    u64,
    *const c_void,
    u64,
) -> *mut u8;

static START: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CONTINUE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static USE_ITEM_BOOL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static USE_ITEM_INTERACTION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ATTACK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SURVIVAL_ATTACK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static USE_ON: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

thread_local! {
    // Set while this thread decides one player action, so a hooked body that calls another raises once.
    static DECIDING: Cell<bool> = const { Cell::new(false) };
}

/// The loader's detours on the `GameMode` slots and `Item::useOn` behind the player-action events.
///
/// Each detour asks the adapter first (inside `Boundary::run`, so a panic counts as "not cancelled") and calls the
/// original unchanged unless a listener cancelled the action. A refused action gets the game's own "not handled"
/// value: `false` (and `hasDestroyedBlock = false` for the destroy calls), the `InteractionResult` the game writes
/// when its own item-use event is cancelled (`{3, 0}`) for the 70.x `useItem`, and a result with no flags for
/// `useOn`.
pub struct PlayerActionDetours;

impl PlayerActionDetours {
    pub const GAME_MODE_VTABLE: &'static str = "GameMode.vtable";
    pub const SURVIVAL_MODE_VTABLE: &'static str = "SurvivalMode.vtable";
    pub const START: &'static str = "GameMode.startDestroyBlock";
    pub const CONTINUE: &'static str = "GameMode.continueDestroyBlock";
    pub const USE_ITEM_BOOL: &'static str = "GameMode.useItem";
    pub const USE_ITEM_INTERACTION: &'static str = "GameMode.useItemInteraction";
    pub const ATTACK: &'static str = "GameMode.attack";
    pub const USE_ON: &'static str = "Item.useOn";
    /// The `InteractionResult` the 70.x `useItem` gets for a refused use: the result value the game writes when its
    /// item-use event is cancelled, then a cleared "stack changed" byte.
    pub const REFUSED_RESULT: i32 = 3;

    pub fn start() -> SessionHook {
        Self::slot(Self::START, Self::GAME_MODE_VTABLE, Self::start_destroy as StartFn as *const c_void, &START)
    }

    pub fn continue_destroy() -> SessionHook {
        Self::slot(
            Self::CONTINUE,
            Self::GAME_MODE_VTABLE,
            Self::continue_destroy_detour as ContinueFn as *const c_void,
            &CONTINUE,
        )
    }

    pub fn use_item_bool() -> SessionHook {
        Self::slot(
            Self::USE_ITEM_BOOL,
            Self::GAME_MODE_VTABLE,
            Self::use_item_bool_detour as UseItemBoolFn as *const c_void,
            &USE_ITEM_BOOL,
        )
    }

    pub fn use_item_interaction() -> SessionHook {
        let detour = Self::use_item_interaction_detour as UseItemInteractionFn as *const c_void;
        Self::slot(Self::USE_ITEM_INTERACTION, Self::GAME_MODE_VTABLE, detour, &USE_ITEM_INTERACTION)
    }

    pub fn attack() -> SessionHook {
        Self::slot(Self::ATTACK, Self::GAME_MODE_VTABLE, Self::attack_detour as AttackFn as *const c_void, &ATTACK)
    }

    pub fn survival_attack() -> SessionHook {
        let detour = Self::survival_attack_detour as AttackFn as *const c_void;
        Self::slot(Self::ATTACK, Self::SURVIVAL_MODE_VTABLE, detour, &SURVIVAL_ATTACK)
    }

    pub fn use_on() -> SessionHook {
        SessionHook {
            entry: Self::USE_ON,
            vtable: None,
            detour: Self::use_on_detour as UseOnFn as *const c_void,
            original: &USE_ON,
        }
    }

    /// Writes the refused result of the 70.x `useItem` to `out`: the result value, then a cleared byte at +4.
    ///
    /// # Safety
    /// `out` is null or writable for 5 bytes.
    pub unsafe fn write_refused_interaction(out: *mut u8) {
        if out.is_null() {
            return;
        }
        // SAFETY: the caller guarantees 5 writable bytes at `out`.
        unsafe {
            out.cast::<i32>().write_unaligned(Self::REFUSED_RESULT);
            out.add(4).write(0);
        }
    }

    /// Writes the refused result of `Item::useOn` to `out`: no flags (no success, no swing).
    ///
    /// # Safety
    /// `out` is null or writable for 1 byte.
    pub unsafe fn write_refused_use_on(out: *mut u8) {
        if !out.is_null() {
            // SAFETY: the caller guarantees the byte is writable; the game's body writes the same byte first.
            unsafe { out.write(0) };
        }
    }

    fn slot(
        entry: &'static str,
        vtable: &'static str,
        detour: *const c_void,
        original: &'static AtomicPtr<c_void>,
    ) -> SessionHook {
        SessionHook { entry, vtable: Some(vtable), detour, original }
    }

    // Runs one decision unless this thread is already deciding one; false (call the original) on re-entry or panic.
    fn decide(body: impl FnOnce(&PlayerActionAdapter) -> bool) -> bool {
        if DECIDING.with(Cell::get) {
            return false;
        }
        DECIDING.with(|flag| flag.set(true));
        let cancelled = Boundary::run(false, || PlayerActionAdapter::with(false, body));
        DECIDING.with(|flag| flag.set(false));
        cancelled
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    pub(crate) unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the caller
        // guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    // `pos` as a value; the origin for a null pointer.
    pub(crate) fn position(pos: *const AmmlBlockPos) -> AmmlBlockPos {
        if pos.is_null() {
            return AmmlBlockPos::default();
        }
        // SAFETY: a non-null `pos` is the game's `BlockPos const&` for this call: three ints.
        unsafe { pos.read_unaligned() }
    }

    unsafe extern "C" fn start_destroy(
        this: *mut c_void,
        pos: *const AmmlBlockPos,
        face: u64,
        destroyed: *mut u8,
    ) -> u8 {
        // SAFETY: the hook on the `GameMode.startDestroyBlock` slot is installed, so its slot holds a `StartFn`.
        let original = unsafe { Self::original::<StartFn>(&START) };
        let cancelled = Self::decide(|adapter| {
            adapter.game_mode_action(this, |local| BlockBreakStart {
                header: PlayerActionAdapter::header::<BlockBreakStart>(),
                pos: Self::position(pos),
                face: amml_abi::AmmlFace(face as u8),
                is_local_player: local,
                _reserved: [0; 2],
            })
        });
        if cancelled {
            if !destroyed.is_null() {
                // SAFETY: `destroyed` is the bool& the caller passed for this call to fill.
                unsafe { destroyed.write(0) };
            }
            return 0;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, pos, face, destroyed) }
    }

    unsafe extern "C" fn continue_destroy_detour(
        this: *mut c_void,
        pos: *const AmmlBlockPos,
        face: u64,
        player_pos: *const c_void,
        destroyed: *mut u8,
    ) -> u8 {
        // SAFETY: the hook on the `GameMode.continueDestroyBlock` slot is installed, so its slot holds a `ContinueFn`.
        let original = unsafe { Self::original::<ContinueFn>(&CONTINUE) };
        let cancelled = Self::decide(|adapter| {
            adapter.game_mode_action(this, |local| BlockBreakContinue {
                header: PlayerActionAdapter::header::<BlockBreakContinue>(),
                pos: Self::position(pos),
                face: amml_abi::AmmlFace(face as u8),
                is_local_player: local,
                _reserved: [0; 2],
            })
        });
        if cancelled {
            if !destroyed.is_null() {
                // SAFETY: `destroyed` is the bool& the caller passed for this call to fill.
                unsafe { destroyed.write(0) };
            }
            return 0;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, pos, face, player_pos, destroyed) }
    }

    fn item_use(adapter: &PlayerActionAdapter, this: *mut c_void) -> bool {
        adapter.game_mode_action(this, |local| ItemUse {
            header: PlayerActionAdapter::header::<ItemUse>(),
            is_local_player: local,
            _reserved: [0; 7],
        })
    }

    unsafe extern "C" fn use_item_bool_detour(this: *mut c_void, stack: *mut c_void, flag: u64) -> u8 {
        // SAFETY: the hook on the `GameMode.useItem` slot is installed, so its slot holds a `UseItemBoolFn`.
        let original = unsafe { Self::original::<UseItemBoolFn>(&USE_ITEM_BOOL) };
        if Self::decide(|adapter| Self::item_use(adapter, this)) {
            return 0;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, stack, flag) }
    }

    unsafe extern "C" fn use_item_interaction_detour(
        this: *mut c_void,
        out: *mut u8,
        stack: *mut c_void,
        flag: u64,
    ) -> *mut u8 {
        // SAFETY: the hook on the `GameMode.useItemInteraction` slot is installed, so its slot holds a
        // `UseItemInteractionFn`.
        let original = unsafe { Self::original::<UseItemInteractionFn>(&USE_ITEM_INTERACTION) };
        if Self::decide(|adapter| Self::item_use(adapter, this)) {
            // SAFETY: `out` is the InteractionResult the caller passed for this call to fill.
            unsafe { Self::write_refused_interaction(out) };
            return out;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, out, stack, flag) }
    }

    fn attack_action(adapter: &PlayerActionAdapter, this: *mut c_void) -> bool {
        adapter.game_mode_action(this, |local| PlayerAttack {
            header: PlayerActionAdapter::header::<PlayerAttack>(),
            is_local_player: local,
            _reserved: [0; 7],
        })
    }

    unsafe extern "C" fn attack_detour(this: *mut c_void, target: *mut c_void, click: *const c_void) -> u8 {
        // SAFETY: the hook on the `GameMode.attack` slot of `GameMode.vtable` is installed, so its slot holds an
        // `AttackFn`.
        let original = unsafe { Self::original::<AttackFn>(&ATTACK) };
        if Self::decide(|adapter| Self::attack_action(adapter, this)) {
            return 0;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, target, click) }
    }

    unsafe extern "C" fn survival_attack_detour(this: *mut c_void, target: *mut c_void, click: *const c_void) -> u8 {
        // SAFETY: the hook on the `GameMode.attack` slot of `SurvivalMode.vtable` is installed, so its slot holds an
        // `AttackFn`.
        let original = unsafe { Self::original::<AttackFn>(&SURVIVAL_ATTACK) };
        if Self::decide(|adapter| Self::attack_action(adapter, this)) {
            return 0;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, target, click) }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn use_on_detour(
        item: *mut c_void,
        result: *mut u8,
        stack: *mut c_void,
        actor: *mut c_void,
        pos: *const AmmlBlockPos,
        face: u64,
        extra: u64,
        click: *const c_void,
        flags: u64,
    ) -> *mut u8 {
        // SAFETY: the hook on `Item.useOn` is installed, so its slot holds a `UseOnFn`.
        let original = unsafe { Self::original::<UseOnFn>(&USE_ON) };
        let cancelled = !result.is_null()
            && Self::decide(|adapter| {
                adapter.actor_action(actor.expose_provenance(), |local| ItemUseOnBlock {
                    header: PlayerActionAdapter::header::<ItemUseOnBlock>(),
                    pos: Self::position(pos),
                    face: amml_abi::AmmlFace(face as u8),
                    is_local_player: local,
                    _reserved: [0; 2],
                })
            });
        if cancelled {
            // SAFETY: `result` is the InteractionResult the caller passed for this call to fill.
            unsafe { Self::write_refused_use_on(result) };
            return result;
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(item, result, stack, actor, pos, face, extra, click, flags) }
    }
}

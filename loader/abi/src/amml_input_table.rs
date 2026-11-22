use crate::{
    AmmlLeaseRequest, AmmlLeaseState, AmmlModifiers, AmmlMouseButton, AmmlMoveIntent, AmmlOtherLeases, AmmlStatus,
    AmmlStr,
};

/// Gives the held modifier keys.
pub type InputModifiersFn = unsafe extern "C" fn() -> AmmlModifiers;
/// Gives 1 in `out` while `key` is down. `NOT_READY` before the first key event.
pub type InputIsDownFn = unsafe extern "C" fn(key: u16, out: *mut u8) -> AmmlStatus;
/// Takes an input lease for `module` and gives its id in `out_lease`.
pub type InputAcquireFn =
    unsafe extern "C" fn(module: AmmlStr, request: *const AmmlLeaseRequest, out_lease: *mut u64) -> AmmlStatus;
/// Gives the state of `lease`; `LOST` for an unknown id.
pub type InputLeaseStateFn = unsafe extern "C" fn(lease: u64) -> AmmlLeaseState;
/// Presses `key` under `lease` and gives the hold in `out_hold`.
pub type InputKeyDownFn = unsafe extern "C" fn(lease: u64, key: u16, out_hold: *mut u64) -> AmmlStatus;
/// Presses `button` under `lease` and gives the hold in `out_hold`.
pub type InputButtonDownFn =
    unsafe extern "C" fn(lease: u64, button: AmmlMouseButton, out_hold: *mut u64) -> AmmlStatus;
/// Releases `hold`. A hold that is already released gives `OK`.
pub type InputReleaseHoldFn = unsafe extern "C" fn(hold: u64) -> AmmlStatus;
/// Scrolls the wheel by `notches` under `lease`.
pub type InputWheelFn = unsafe extern "C" fn(lease: u64, notches: f32) -> AmmlStatus;
/// Turns the view by `yaw` and `pitch` under `lease`.
pub type InputTurnFn = unsafe extern "C" fn(lease: u64, yaw: f32, pitch: f32) -> AmmlStatus;
/// Sets the movement of `lease`; a null `intent` means no intent.
pub type InputSetMoveFn = unsafe extern "C" fn(lease: u64, intent: *const AmmlMoveIntent) -> AmmlStatus;
/// Releases every hold of `lease`.
pub type InputReleaseAllFn = unsafe extern "C" fn(lease: u64);
/// Ends `lease` and releases its holds.
pub type InputEndLeaseFn = unsafe extern "C" fn(lease: u64) -> AmmlStatus;
/// Sets the lease ranking of `module`: `count` mod ids in `ids`, highest first.
pub type InputSetRankingFn = unsafe extern "C" fn(module: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus;
/// Sets the coexist pairs of `module`: `count` mod ids in `ids`, two per pair, so `count` is even.
pub type InputSetCoexistFn = unsafe extern "C" fn(module: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus;
/// Gives in `out` the active input leases of every live mod but `module` (the caller never counts): how many, and the
/// union of the real input they block. A suspended lease does not count. `INVALID_ARGUMENT` for a module that is not a
/// live mod.
pub type InputHeldByOthersFn = unsafe extern "C" fn(module: AmmlStr, out: *mut AmmlOtherLeases) -> AmmlStatus;

/// The input functions of `AmmlServices`.
///
/// It only grows. A table from an older loader can be shorter than this struct: read it with
/// `AmmlInputTable::read`, which leaves the fields its `size` does not cover at `None`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlInputTable {
    pub size: u32,
    pub version: u32,
    pub modifiers: InputModifiersFn,
    pub is_down: InputIsDownFn,
    pub acquire: InputAcquireFn,
    pub lease_state: InputLeaseStateFn,
    pub key_down: InputKeyDownFn,
    pub button_down: InputButtonDownFn,
    pub release_hold: InputReleaseHoldFn,
    pub wheel: InputWheelFn,
    pub turn: InputTurnFn,
    pub set_move: InputSetMoveFn,
    pub release_all: InputReleaseAllFn,
    pub end_lease: InputEndLeaseFn,
    pub set_ranking: InputSetRankingFn,
    /// From v2.
    pub set_coexist: Option<InputSetCoexistFn>,
    /// From v3.
    pub held_by_others: Option<InputHeldByOthersFn>,
}

impl AmmlInputTable {
    /// `size` of a v1 input table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 112;
    /// `size` of a v2 input table: v1 plus `set_coexist`.
    pub const SIZE_V2: u32 = 120;
    /// `size` of a v3 input table: v2 plus `held_by_others`.
    pub const SIZE_V3: u32 = 128;

    /// Copies the table at `table`: the bytes its `size` covers, with every later field `None`. A null
    /// table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlInputTable) -> Option<AmmlInputTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlInputTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlInputTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the
        // whole struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlInputTable>() == AmmlInputTable::SIZE_V3 as usize);

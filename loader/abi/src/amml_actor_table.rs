use core::ffi::c_void;

use crate::{AmmlRotation, AmmlStatus, AmmlVec3};

/// Gives 1 in `out` while the actor `player` is alive, 0 when it is dead. `player` is a local player handle that
/// a client call gave in the same game frame.
pub type ActorIsAliveFn = unsafe extern "C" fn(player: *const c_void, out: *mut u8) -> AmmlStatus;

/// Gives a point of the actor `player` in `out` (the feet for `position`, the eyes for `eye_position`, the head for
/// `head_position`). Area `actor.read` version 2; `head_position` is version 3.
pub type ActorPointFn = unsafe extern "C" fn(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus;
/// Gives the view direction of the actor `player` in `out`. Area `actor.read` version 2.
pub type ActorRotationFn = unsafe extern "C" fn(player: *const c_void, out: *mut AmmlRotation) -> AmmlStatus;
/// Gives the dimension of the actor `player` in `out`, as a handle for `dimension_id`. Area `actor.read` version 2.
pub type ActorDimensionFn = unsafe extern "C" fn(player: *const c_void, out: *mut *const c_void) -> AmmlStatus;
/// Gives the id of `dimension`, a handle `dimension` gave in the same frame. Area `actor.read` version 2.
pub type DimensionIdFn = unsafe extern "C" fn(dimension: *const c_void, out: *mut i32) -> AmmlStatus;

/// Gives the block source of the dimension of the actor `player` in `out`, as a handle for the world calls. Area
/// `actor.read` version 3.
pub type ActorBlockSourceFn = unsafe extern "C" fn(player: *const c_void, out: *mut *const c_void) -> AmmlStatus;
/// Gives the health of the actor `player` in `out`. Area `actor.health`.
pub type ActorHealthFn = unsafe extern "C" fn(player: *const c_void, out: *mut i32) -> AmmlStatus;

/// The actor functions of `AmmlServices` (area `actor.read`). The table only grows: a function added later is
/// appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlActorTable {
    pub size: u32,
    pub version: u32,
    pub is_alive: ActorIsAliveFn,
    pub position: Option<ActorPointFn>,
    pub eye_position: Option<ActorPointFn>,
    pub rotation: Option<ActorRotationFn>,
    pub dimension: Option<ActorDimensionFn>,
    pub dimension_id: Option<DimensionIdFn>,
    pub head_position: Option<ActorPointFn>,
    pub block_source: Option<ActorBlockSourceFn>,
    pub health: Option<ActorHealthFn>,
}

impl AmmlActorTable {
    /// `size` of a v1 actor table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 16;
    /// `size` of a v2 actor table: v1 plus `position`, `eye_position`, `rotation`, `dimension` and `dimension_id`.
    pub const SIZE_V2: u32 = 56;
    /// `size` of a v3 actor table: v2 plus `head_position`, `block_source` and `health`.
    pub const SIZE_V3: u32 = 80;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null
    /// table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlActorTable) -> Option<AmmlActorTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlActorTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlActorTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlActorTable>() == AmmlActorTable::SIZE_V3 as usize);

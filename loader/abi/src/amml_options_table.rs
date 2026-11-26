use core::ffi::c_void;

use crate::{AmmlOptionKey, AmmlPersistence, AmmlStatus, AmmlStr};

/// Gives in `out` the view distance of the client `client` in chunks (`IOptionRegistry::getViewDistanceChunks`). Game
/// thread only, for the primary client (`REFUSED` elsewhere); `NOT_READY` before the first frame; `NOT_AVAILABLE`
/// when the area is not offered. Area `options.view_distance`.
pub type OptionsViewDistanceFn = unsafe extern "C" fn(client: *const c_void, out: *mut u32) -> AmmlStatus;

/// Gives in `out` the value of the integer option `key`. Game thread only (`REFUSED` elsewhere); `NOT_READY` before
/// the first frame or before a client exists; `NOT_AVAILABLE` when the area is not offered or the loader found no
/// option for `key`; `INVALID_ARGUMENT` for an unknown key or a null `out`. Area `options.int`.
pub type OptionsGetIntFn = unsafe extern "C" fn(key: AmmlOptionKey, out: *mut i32) -> AmmlStatus;

/// Sets the integer option `key` to `value` for the mod `module`. With `AmmlPersistence::TRANSIENT` the game is told
/// not to save the change, and the loader keeps the value from before the first transient set of the key: it writes it
/// back and saves it at `release_transient` or at the mod's disable, but only while the option still holds the value
/// this mod set. `REFUSED` when another mod holds a transient value of the key, and off the game thread;
/// `INVALID_ARGUMENT` for an unknown key or persistence; `NOT_AVAILABLE` and `NOT_READY` as for `get_int`. Area
/// `options.int`.
pub type OptionsSetIntFn =
    unsafe extern "C" fn(module: AmmlStr, key: AmmlOptionKey, value: i32, persistence: AmmlPersistence) -> AmmlStatus;

/// Ends the transient value of `key` that the mod `module` holds: the value from before its first transient set is
/// written back and saved when the option still holds the transient value, and left alone when the user changed it.
/// `OK` with nothing written when no mod holds a transient value of the key; `REFUSED` when another mod holds it, and
/// off the game thread. Area `options.int`.
pub type OptionsReleaseTransientFn = unsafe extern "C" fn(module: AmmlStr, key: AmmlOptionKey) -> AmmlStatus;

/// The option functions of `AmmlServices` (areas `options.*`). The table only grows: a function added later is
/// appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlOptionsTable {
    pub size: u32,
    pub version: u32,
    pub view_distance_chunks: OptionsViewDistanceFn,
    pub get_int: OptionsGetIntFn,
    pub set_int: OptionsSetIntFn,
    pub release_transient: OptionsReleaseTransientFn,
}

impl AmmlOptionsTable {
    /// `size` of a v1 options table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 40;

    /// A copy of the table at `table`, read as far as its own `size` covers. A null table, or one shorter than v1,
    /// gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlOptionsTable) -> Option<AmmlOptionsTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlOptionsTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlOptionsTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole struct
        // and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: `size` covers every v1 field, so the copy holds a valid table.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlOptionsTable>() == AmmlOptionsTable::SIZE_V1 as usize);

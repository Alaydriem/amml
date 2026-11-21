use core::ffi::c_void;

use crate::{AmmlStatus, AmmlStr};

/// Publishes `table` as the provider of `name` at `major.minor`.
pub type ProvideFn = unsafe extern "C" fn(name: AmmlStr, major: u16, minor: u16, table: *const c_void) -> AmmlStatus;
/// Looks up the provider of `name`. Gives `NOT_AVAILABLE` when there is none for `major` with a
/// minor of at least `min_minor`.
pub type RequireFn = unsafe extern "C" fn(
    name: AmmlStr,
    major: u16,
    min_minor: u16,
    out_table: *mut *const c_void,
    out_minor: *mut u16,
) -> AmmlStatus;

/// The provider registry functions of `AmmlServices`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlRegistryTable {
    pub size: u32,
    pub version: u32,
    pub provide: ProvideFn,
    pub require: RequireFn,
}

impl AmmlRegistryTable {
    /// `size` of a v1 registry table. A literal, so it keeps its meaning when a later version adds
    /// fields: a reader accepts any `size` at or above it and reads a later field only when `size`
    /// covers it.
    pub const SIZE_V1: u32 = 24;
}

const _: () = assert!(core::mem::size_of::<AmmlRegistryTable>() == AmmlRegistryTable::SIZE_V1 as usize);

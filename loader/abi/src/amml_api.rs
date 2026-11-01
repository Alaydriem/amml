use core::ffi::c_void;

use crate::{AmmlLogLevel, AmmlStatus, AmmlStr};

/// Version of the `AmmlApi` table this crate describes.
pub const AMML_API_VERSION: u32 = 1;

/// Resolves an entry id to its value (RVA, slot index or byte offset) in `out`.
pub type ResolveFn = unsafe extern "C" fn(id: AmmlStr, out: *mut u64) -> AmmlStatus;
/// Installs a detour on `target`; `original` receives the next function in the chain.
///
/// Hooks on one target run in this order: a higher `priority` runs first, and hooks of equal
/// priority run in the order they were registered.
///
/// The loader accepts a hook only while it is inside the mod's `amml_mod_load` or `amml_mod_enable`
/// call and returns `FAILED` otherwise, in `amml_mod_disable` and `amml_mod_unload` too. First hooks
/// on a target patch its prologue and must happen before the game runs it.
pub type HookFn = unsafe extern "C" fn(
    target: *const c_void,
    detour: *const c_void,
    priority: i32,
    original: *mut *const c_void,
    handle: *mut u64,
) -> AmmlStatus;
/// Removes the hook behind `handle`.
///
/// A mod can remove only a hook it installed itself, and only while the loader is inside one of its
/// callbacks; any other call returns `FAILED`.
pub type UnhookFn = unsafe extern "C" fn(handle: u64) -> AmmlStatus;
/// Writes one log line for `module`.
pub type LogFn = unsafe extern "C" fn(module: AmmlStr, level: AmmlLogLevel, message: AmmlStr);
/// Allocates `size` bytes on the game heap. The result is null only if the game allocator returns null.
pub type HeapNewFn = unsafe extern "C" fn(usize) -> *mut u8;
/// Frees a block from `HeapNewFn` on the game heap. The size must be the one given to `HeapNewFn`.
pub type HeapDeleteFn = unsafe extern "C" fn(*mut u8, usize);
/// Gives a mod folder (`kind` 0), config folder (1) or data folder (2) in `out`.
///
/// The loader owns the bytes written to `*out`; they stay valid until `amml_mod_unload` returns.
/// Nothing is written on error. An unknown `kind` returns `INVALID_ARGUMENT`.
pub type ModDirFn = unsafe extern "C" fn(module: AmmlStr, kind: u8, out: *mut AmmlStr) -> AmmlStatus;

/// The function table the loader hands to a mod (v1).
///
/// A mod accepts a table whose `version` is at least its own and reads only fields that fit within `size`.
///
/// A table from an older loader can be shorter than this struct. A mod must read `size` and
/// `version` first through raw-pointer field reads (`AmmlApi::header`) and must never form an
/// `&AmmlApi` unless `size >= AmmlApi::SIZE_V1`.
#[repr(C)]
pub struct AmmlApi {
    pub size: u32,
    pub version: u32,
    pub module_base: u64,
    pub resolve: ResolveFn,
    pub hook: HookFn,
    pub unhook: UnhookFn,
    pub log: LogFn,
    pub heap_new: HeapNewFn,
    pub heap_delete: HeapDeleteFn,
    pub mod_dir: ModDirFn,
}

impl AmmlApi {
    /// `size` of a v1 table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 72;

    /// Reads `(size, version)` through a raw pointer, touching only the two leading fields.
    /// Returns `None` for a null pointer.
    ///
    /// # Safety
    /// A non-null `api` must point to at least 8 readable bytes, suitably aligned.
    #[allow(unsafe_code)]
    pub unsafe fn header(api: *const AmmlApi) -> Option<(u32, u32)> {
        if api.is_null() {
            return None;
        }
        // SAFETY: `api` is non-null and the caller guarantees the two leading u32 fields are readable.
        let size = unsafe { core::ptr::addr_of!((*api).size).read() };
        // SAFETY: as above.
        let version = unsafe { core::ptr::addr_of!((*api).version).read() };
        Some((size, version))
    }

    /// Whether the field at `field_offset` with `field_size` bytes lies entirely within `size`.
    ///
    /// Valid only on a full-size table (`size >= SIZE_V1`); read `header` first on anything shorter.
    pub fn supports(&self, field_offset: usize, field_size: usize) -> bool {
        match field_offset.checked_add(field_size) {
            Some(end) => end <= self.size as usize,
            None => false,
        }
    }
}

const _: () = assert!(core::mem::size_of::<AmmlApi>() == AmmlApi::SIZE_V1 as usize);

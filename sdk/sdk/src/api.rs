#![allow(unsafe_code)]

use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use amml_abi::{
    AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr, HeapDeleteFn, HeapNewFn, HookFn, LogFn, ModDirFn, ResolveFn, UnhookFn,
};

use crate::{HookHandle, SdkError};

/// The loader function table, checked and copied. Cheap to copy; it holds function pointers only.
#[derive(Clone, Copy)]
pub struct Api {
    module_base: u64,
    resolve: ResolveFn,
    hook: HookFn,
    unhook: UnhookFn,
    log: LogFn,
    heap_new: HeapNewFn,
    heap_delete: HeapDeleteFn,
    mod_dir: ModDirFn,
}

impl Api {
    /// Checks and copies the table the loader passed to `amml_mod_load`. The table must hold every
    /// v1 field and its `version` must be at least `required_version`, the lowest loader API the
    /// mod accepts (`AmmlMod::API_VERSION`).
    ///
    /// # Safety
    /// A non-null `api` must point to a table of at least `size` bytes, as the loader promises.
    pub unsafe fn from_raw(api: *const AmmlApi, required_version: u32) -> Result<Self, SdkError> {
        // SAFETY: the caller guarantees a non-null `api` has its two leading fields readable.
        let Some((size, version)) = (unsafe { AmmlApi::header(api) }) else {
            return Err(SdkError::NullApi);
        };
        if size < AmmlApi::SIZE_V1 {
            return Err(SdkError::TableTooSmall(size));
        }
        if version < required_version {
            return Err(SdkError::LoaderTooOld(version));
        }
        // SAFETY: `size` covers the whole v1 table, so a reference to it is valid.
        let table = unsafe { &*api };
        Ok(Self {
            module_base: table.module_base,
            resolve: table.resolve,
            hook: table.hook,
            unhook: table.unhook,
            log: table.log,
            heap_new: table.heap_new,
            heap_delete: table.heap_delete,
            mod_dir: table.mod_dir,
        })
    }

    /// Base address of the game module the entries are relative to.
    pub fn module_base(&self) -> u64 {
        self.module_base
    }

    /// The value of an entry: an absolute address for `function` and `global` entries, a slot index
    /// for `vslot` and a byte offset for `field`.
    pub fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        let mut value = 0u64;
        // SAFETY: the loader reads `id` for the call and writes one u64 through `out`.
        let status = unsafe { (self.resolve)(AmmlStr::from_str(id), &mut value) };
        match status {
            AmmlStatus::OK => Ok(value),
            AmmlStatus::MISSING_ENTRY => Err(SdkError::MissingEntry(id.to_string())),
            other => Err(SdkError::Status(other)),
        }
    }

    /// Installs `detour` on `target`. A higher `priority` runs first; hooks of equal priority run in
    /// the order they were registered. The loader writes the next function in the chain into
    /// `original` before the detour can run; call it through that slot.
    ///
    /// The loader accepts a hook only from `AmmlMod::load` and `AmmlMod::enable`; in `disable` and
    /// `unload` it returns an error.
    ///
    /// # Safety
    /// `target` must be the start of a function the loader can detour, and `detour` an `extern "C"`
    /// function with the same signature. `original` must be used for this hook only.
    pub unsafe fn hook(
        &self,
        target: u64,
        detour: *const c_void,
        priority: i32,
        original: &'static AtomicPtr<c_void>,
    ) -> Result<HookHandle, SdkError> {
        let mut handle = 0u64;
        // SAFETY: the caller guarantees the target and detour; `original` is a static, aligned
        // pointer slot and `handle` is a live u64.
        let status = unsafe {
            (self.hook)(
                target as usize as *const c_void,
                detour,
                priority,
                original.as_ptr().cast::<*const c_void>(),
                &mut handle,
            )
        };
        match status {
            AmmlStatus::OK => Ok(HookHandle(handle)),
            other => Err(SdkError::Status(other)),
        }
    }

    /// Removes a hook this mod installed. The loader refuses a handle of another mod, and a call
    /// from outside a mod callback.
    pub fn unhook(&self, handle: HookHandle) -> Result<(), SdkError> {
        // SAFETY: the loader accepts any handle value and reports an unknown one as a status.
        let status = unsafe { (self.unhook)(handle.0) };
        match status {
            AmmlStatus::OK => Ok(()),
            other => Err(SdkError::Status(other)),
        }
    }

    /// Writes one line to the log of `module`.
    pub fn log(&self, module: &str, level: AmmlLogLevel, message: &str) {
        // SAFETY: both views live for the call, and the loader copies what it keeps.
        unsafe { (self.log)(AmmlStr::from_str(module), level, AmmlStr::from_str(message)) }
    }

    /// Allocates on the game heap. Null when the loader has no game allocator or the game returns null.
    pub fn heap_new(&self, size: usize) -> *mut u8 {
        // SAFETY: the loader takes any size and returns null on failure.
        unsafe { (self.heap_new)(size) }
    }

    /// Frees a block from `heap_new`.
    ///
    /// # Safety
    /// `ptr` must come from `heap_new` with the same `size` and must not be freed twice.
    pub unsafe fn heap_delete(&self, ptr: *mut u8, size: usize) {
        // SAFETY: the caller guarantees `ptr` and `size`.
        unsafe { (self.heap_delete)(ptr, size) }
    }

    /// Asks for a folder of `module`: `kind` 0 mod folder, 1 config folder, 2 data folder, 3 shared data
    /// folder (`INVALID_ARGUMENT` from a loader that has none).
    pub fn mod_dir(&self, module: &str, kind: u8) -> Result<String, SdkError> {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `module` lives for the call and `out` is a live AmmlStr.
        let status = unsafe { (self.mod_dir)(AmmlStr::from_str(module), kind, &mut out) };
        if !status.is_ok() {
            return Err(SdkError::Status(status));
        }
        // SAFETY: the loader keeps the bytes valid until `amml_mod_unload` returns; this copies them.
        let text = unsafe { out.as_str() }.map_err(|_| SdkError::InvalidString)?;
        Ok(text.to_string())
    }
}

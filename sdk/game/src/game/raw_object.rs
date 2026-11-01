use std::ffi::c_void;

/// Raw reads of game objects and the calls of game code, for the typed wrappers.
///
/// Every function here trusts its caller that the pointer is a live game object of the class the entry values
/// were derived for. The wrappers check each pointer for null before they pass it on.
pub struct RawObject;

impl RawObject {
    /// The code pointer in `slot` of the vtable at the start of `object`; `None` for a null vtable or slot.
    ///
    /// # Safety
    /// `object` must point to a live object whose first qword is a vtable with at least `slot + 1` entries.
    #[allow(unsafe_code)]
    pub unsafe fn slot(object: *const c_void, slot: u64) -> Option<*const c_void> {
        let slot = usize::try_from(slot).ok()?;
        // SAFETY: the caller guarantees `object` is readable for a qword.
        let vtable = unsafe { *object.cast::<*const *const c_void>() };
        if vtable.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the vtable holds `slot`.
        let function = unsafe { *vtable.add(slot) };
        (!function.is_null()).then_some(function)
    }

    /// A `T` at `offset` bytes into `base`; `None` when the offset does not fit `usize`.
    ///
    /// # Safety
    /// `base + offset` must be `size_of::<T>()` readable bytes of a live object.
    #[allow(unsafe_code)]
    pub unsafe fn read<T: Copy>(base: *const c_void, offset: u64) -> Option<T> {
        let offset = usize::try_from(offset).ok()?;
        // SAFETY: the caller guarantees the bytes are readable; the read does not need alignment.
        Some(unsafe { base.cast::<u8>().add(offset).cast::<T>().read_unaligned() })
    }

    /// A pointer field at `offset` bytes into `base`; `None` when it is null.
    ///
    /// # Safety
    /// As [`RawObject::read`] for a pointer.
    #[allow(unsafe_code)]
    pub unsafe fn pointer(base: *const c_void, offset: u64) -> Option<*mut c_void> {
        // SAFETY: forwarded to the caller.
        let pointer = unsafe { Self::read::<*mut c_void>(base, offset) }?;
        (!pointer.is_null()).then_some(pointer)
    }

    /// Calls `T* f(this)` (a pointer or reference return); `None` for a null function or a null result.
    ///
    /// # Safety
    /// `function` must be game code with that signature and `this` a valid object for it.
    #[allow(unsafe_code)]
    pub unsafe fn call_pointer(function: *const c_void, this: *mut c_void) -> Option<*mut c_void> {
        if function.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the signature.
        let call: unsafe extern "C" fn(*mut c_void) -> *mut c_void = unsafe { std::mem::transmute(function) };
        // SAFETY: the caller guarantees `this` is valid for the function.
        let result = unsafe { call(this) };
        (!result.is_null()).then_some(result)
    }

    /// Calls `bool f(this)`, read as a byte; `false` for a null function.
    ///
    /// # Safety
    /// As [`RawObject::call_pointer`], for a `bool` return.
    #[allow(unsafe_code)]
    pub unsafe fn call_bool(function: *const c_void, this: *mut c_void) -> bool {
        if function.is_null() {
            return false;
        }
        // SAFETY: the caller guarantees the signature.
        let call: unsafe extern "C" fn(*mut c_void) -> u8 = unsafe { std::mem::transmute(function) };
        // SAFETY: the caller guarantees `this` is valid for the function.
        unsafe { call(this) != 0 }
    }
}

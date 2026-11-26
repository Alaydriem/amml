use std::ffi::c_void;

/// Layout mirror of the MSVC `std::function` object (64 bytes on x64): seven words of small-object space, then the
/// pointer to the callable object (`_Func_base`, null for an empty function) in the last word. The callable's
/// `_Do_call` is slot `DO_CALL_SLOT` of its vtable.
#[repr(C)]
pub struct FunctionStorage {
    pub space: [*mut c_void; 7],
    pub callable: *mut c_void,
}

impl FunctionStorage {
    /// The slot of `_Do_call` in the vtable of the callable: after `_Copy` and `_Move`.
    pub const DO_CALL_SLOT: usize = 2;
}

const _: () = assert!(std::mem::offset_of!(FunctionStorage, callable) == 0x38);
const _: () = assert!(std::mem::size_of::<FunctionStorage>() == 64);

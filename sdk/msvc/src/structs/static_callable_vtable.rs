use std::ffi::c_void;

/// The six virtual functions of the MSVC `_Func_base` of a `std::function<void(A)>`, in slot order: `_Copy`,
/// `_Move`, `_Do_call`, `_Target_type`, `_Delete_this`, `_Get`. Built by `new` for a `StaticCallable`: the callable
/// has no state, so a copy or a move is the same object, and a delete frees nothing.
#[repr(C)]
pub struct StaticCallableVtable {
    copy: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
    moved: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
    do_call: unsafe extern "C" fn(*mut c_void, *const c_void),
    target_type: unsafe extern "C" fn(*mut c_void) -> *const c_void,
    delete_this: unsafe extern "C" fn(*mut c_void, bool),
    get: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
}

// The identity `_Target_type` gives: an empty MSVC `type_info` (vtable and spare word zero, the decorated name an
// empty string at +10h).
static TARGET_TYPE: [u8; 0x20] = [0; 0x20];

impl StaticCallableVtable {
    /// The table of a callable whose `_Do_call` is `do_call`. `_Do_call` gets the callable and a pointer to the one
    /// argument (the game passes it by `&&`). It may run on any thread of the game and must not unwind.
    pub const fn new(do_call: unsafe extern "C" fn(*mut c_void, *const c_void)) -> Self {
        Self {
            copy: Self::same,
            moved: Self::same,
            do_call,
            target_type: Self::target_type,
            delete_this: Self::delete_this,
            get: Self::get,
        }
    }

    unsafe extern "C" fn same(this: *mut c_void, _place: *mut c_void) -> *mut c_void {
        this
    }

    unsafe extern "C" fn target_type(_this: *mut c_void) -> *const c_void {
        TARGET_TYPE.as_ptr().cast()
    }

    unsafe extern "C" fn delete_this(_this: *mut c_void, _dealloc: bool) {}

    unsafe extern "C" fn get(this: *mut c_void) -> *mut c_void {
        this
    }
}

use std::ffi::c_void;

use crate::structs::callback_guard::CallbackGuard;
use crate::structs::vtable::RustObject;

/// The pointers of the MSVC `std::function` storage: seven words of small-object space, then the pointer to the
/// callable object (`_Func_base`) in the last word. 64 bytes on x64.
#[repr(C)]
struct FunctionStorage {
    space: [*mut c_void; 7],
    callable: *mut c_void,
}

/// The virtual functions of the MSVC `_Func_base` in slot order: `_Copy`, `_Move`, `_Do_call`, then `_Target_type`
/// (built with RTTI) or `_Delete_this` (without), then the rest. Only `_Do_call` may run: the holder of an
/// `MsvcFunction` passes it by `const&` and never copies, moves or destroys it, so every other slot aborts.
#[repr(C)]
pub struct FunctionVtable {
    copy: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    moved: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    do_call: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void, *const bool) -> bool,
    fourth: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    fifth: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    sixth: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
}

/// An MSVC `std::function<bool(A const&, B const&, bool)>` that calls a Rust closure, for a C++ function that takes
/// the `std::function` by `const&` and only calls it.
///
/// The closure gets the two references as pointers and the bool by value (`_Do_call` takes every argument by
/// reference). The object is valid while the `MsvcFunction` lives; it may move, the C++ object does not.
pub struct MsvcFunction<F> {
    storage: Box<FunctionStorage>,
    // owns the object that `storage` points to
    _callable: Box<RustObject<FunctionVtable, F>>,
}

impl<F: FnMut(*const c_void, *const c_void, bool) -> bool> MsvcFunction<F> {
    const VTABLE: FunctionVtable = FunctionVtable {
        copy: Self::unexpected,
        moved: Self::unexpected,
        do_call: Self::do_call,
        fourth: Self::unexpected,
        fifth: Self::unexpected,
        sixth: Self::unexpected,
    };

    pub fn new(closure: F) -> Self {
        let mut callable = Box::new(RustObject::new(&Self::VTABLE, closure));
        let callable_ptr = (&raw mut *callable).cast::<c_void>();
        let storage = Box::new(FunctionStorage { space: [std::ptr::null_mut(); 7], callable: callable_ptr });
        Self { storage, _callable: callable }
    }

    /// The address of the C++ `std::function` object, to pass where the game takes `std::function const&`.
    pub fn as_ptr(&self) -> *const c_void {
        (&raw const *self.storage).cast()
    }

    unsafe extern "C" fn do_call(
        this: *mut c_void,
        first: *const c_void,
        second: *const c_void,
        flag: *const bool,
    ) -> bool {
        CallbackGuard::run(|| {
            // SAFETY: `this` is the callable object `new` stored, whose state is the closure.
            let closure = unsafe { &mut *RustObject::<FunctionVtable, F>::state(this.cast()) };
            // SAFETY: `_Do_call` passes the bool by reference; C++ guarantees it points to a live bool.
            let flag = !flag.is_null() && unsafe { *flag };
            closure(first, second, flag)
        })
    }

    unsafe extern "C" fn unexpected(_this: *mut c_void) -> *mut c_void {
        std::process::abort()
    }
}

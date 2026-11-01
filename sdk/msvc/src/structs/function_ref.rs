use std::ffi::c_void;
use std::marker::PhantomData;

use crate::structs::callback_guard::CallbackGuard;

/// The game's `brstd::function_ref<bool(T&, U*, V*)>`: the call thunk, then the object pointer. The thunk takes the
/// object pointer first, then the reference as a pointer and the two pointers by value.
///
/// It borrows the closure: the game must not keep it past the call it was given to. MSVC passes this 16-byte object
/// by a pointer to a copy, so pass [`FunctionRef::as_ptr`].
#[repr(C)]
pub struct FunctionRef<'a, F> {
    thunk: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> bool,
    object: *mut c_void,
    closure: PhantomData<&'a mut F>,
}

impl<'a, F: FnMut(*mut c_void, *mut c_void, *mut c_void) -> bool> FunctionRef<'a, F> {
    pub fn new(closure: &'a mut F) -> Self {
        Self { thunk: Self::thunk, object: (closure as *mut F).cast(), closure: PhantomData }
    }

    /// The address of the object, for a C++ parameter of this type.
    pub fn as_ptr(&self) -> *const c_void {
        (self as *const Self).cast()
    }

    unsafe extern "C" fn thunk(
        object: *mut c_void,
        first: *mut c_void,
        second: *mut c_void,
        third: *mut c_void,
    ) -> bool {
        CallbackGuard::run(|| {
            // SAFETY: `object` is the closure `new` borrowed for `'a`, and the game calls the thunk only during the
            // call the reference was passed to.
            let closure = unsafe { &mut *object.cast::<F>() };
            closure(first, second, third)
        })
    }
}

use std::ffi::c_void;
use std::mem::MaybeUninit;

/// Calls C++ member functions that return a struct by value.
///
/// x64 Windows has one calling convention, and a member function takes `this` as its first argument. A
/// NON-STATIC member function that returns a class, struct or union returns it through a hidden pointer to
/// caller-owned memory, whatever its size or triviality: even an 8-byte trivially copyable `struct { int a,
/// b; }` goes through the pointer. (Only free and static functions return a small trivially copyable struct in
/// RAX.) The pointer is the SECOND argument, after `this` (for a free function it is the first), and the
/// function returns the same pointer in RAX. So `Big Thing::make(int)` is called as `make(this, &mut out, x)`.
///
/// `R` must be a class, struct or union type. A scalar return (`int`, `float`, a pointer) is not returned
/// through a hidden pointer, so it must not go through `MemberCall`.
///
/// The callee must not throw: an exception cannot unwind through these `extern "C"` calls.
///
/// The extra arguments `A` must be passed in one register: an integer, pointer, bool or float of at most 8
/// bytes. A struct argument that is not 1, 2, 4 or 8 bytes is passed by pointer instead and is not supported.
///
/// `func` is the code address of a non-virtual member function. The fixture shows how to get one in C++: a
/// member-function pointer to a non-virtual function of a single-inheritance class is one code pointer.
pub struct MemberCall;

impl MemberCall {
    /// Calls `R f(this)`.
    ///
    /// # Safety
    /// `func` must be the address of a member function with that signature and `this` a valid object for it.
    /// `R` must have the layout of the C++ return type.
    pub unsafe fn call_sret0<R>(func: *const c_void, this: *mut c_void) -> R {
        // SAFETY: the caller guarantees `func` has this signature; sret member calls take (this, out).
        let f: unsafe extern "C" fn(*mut c_void, *mut R) -> *mut R = unsafe { std::mem::transmute(func) };
        let mut out = MaybeUninit::<R>::uninit();
        // SAFETY: `out` has the size and alignment of `R`; the callee constructs the result in it.
        unsafe { f(this, out.as_mut_ptr()) };
        // SAFETY: the callee initialised `out` through the hidden return pointer.
        unsafe { out.assume_init() }
    }

    /// Calls `R f(this, arg)`.
    ///
    /// # Safety
    /// As [`MemberCall::call_sret0`]; `A` must match the C++ parameter type and be passed in one register.
    pub unsafe fn call_sret1<R, A>(func: *const c_void, this: *mut c_void, arg: A) -> R {
        // SAFETY: the caller guarantees `func` has this signature; sret member calls take (this, out, arg).
        let f: unsafe extern "C" fn(*mut c_void, *mut R, A) -> *mut R = unsafe { std::mem::transmute(func) };
        let mut out = MaybeUninit::<R>::uninit();
        // SAFETY: `out` has the size and alignment of `R`; the callee constructs the result in it.
        unsafe { f(this, out.as_mut_ptr(), arg) };
        // SAFETY: the callee initialised `out` through the hidden return pointer.
        unsafe { out.assume_init() }
    }
}

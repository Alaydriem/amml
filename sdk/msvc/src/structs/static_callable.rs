use std::ffi::c_void;

use crate::structs::StaticCallableVtable;

/// A callable with no state in static memory, for a C++ function that takes a `std::function<void(A)>` by value
/// and may copy, move, call and destroy it on any thread (a join result callback).
///
/// `function_block` gives the 64-byte MSVC `std::function` object. The callable is not in the block's small-object
/// space, so the game treats it as a heap callable: it takes the pointer, copies it through `_Copy` (the same
/// object) and ends with `_Delete_this` (nothing freed). The block is never empty, so a call without an emptiness
/// check does not throw.
#[repr(C)]
pub struct StaticCallable {
    vtable: *const StaticCallableVtable,
}

// SAFETY: the object holds only a pointer to an immutable static table and is never written.
#[allow(unsafe_code)]
unsafe impl Sync for StaticCallable {}

impl StaticCallable {
    /// `sizeof(std::function)` under MSVC on x64: seven words of small-object space and the callable pointer.
    pub const FUNCTION_SIZE: usize = 0x40;

    pub const fn new(vtable: &'static StaticCallableVtable) -> Self {
        Self { vtable }
    }

    /// A new `std::function` object whose callable is `self`, to pass where the game takes the function by value
    /// (a pointer to the block).
    pub fn function_block(&'static self) -> [*mut c_void; 8] {
        let mut block = [std::ptr::null_mut(); 8];
        block[7] = std::ptr::from_ref(self).cast_mut().cast();
        block
    }
}

const _: () = assert!(std::mem::size_of::<[*mut c_void; 8]>() == StaticCallable::FUNCTION_SIZE);

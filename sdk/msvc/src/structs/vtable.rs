use crate::errors::MsvcError;
use crate::structs::heap::MsvcHeap;
use std::ffi::c_void;

/// Alignment `operator new` guarantees for small blocks.
const HEAP_ALIGNMENT: usize = 16;

/// Slot 0 of an MSVC class with a virtual destructor: the scalar deleting destructor.
///
/// Bit 0 of `flags` asks the destructor to free the object's memory after destroying it. It returns `this`.
pub type ScalarDeletingDestructor = unsafe extern "C" fn(this: *mut c_void, flags: u32) -> *mut c_void;

/// A Rust object that C++ can use through a virtual interface.
///
/// The first field is the vtable pointer, which is where MSVC puts it. `V` is a `#[repr(C)]` struct of
/// `unsafe extern "C" fn` pointers declared in slot order; for a class with a virtual destructor, slot 0 is
/// a [`ScalarDeletingDestructor`]. Every function takes the object as its first argument (`this`).
///
/// `V` has no RTTI slot at index -1: MSVC reads the complete object locator there for `dynamic_cast` and
/// `typeid`, and here it would read whatever precedes the Rust static. Such an object must never reach code
/// that runs `dynamic_cast` or `typeid` on it.
#[repr(C)]
pub struct RustObject<V: 'static, S> {
    vtable: &'static V,
    state: S,
}

impl<V: 'static, S> RustObject<V, S> {
    const FITS_HEAP_ALIGNMENT: () = assert!(align_of::<Self>() <= HEAP_ALIGNMENT);

    pub fn new(vtable: &'static V, state: S) -> Self {
        Self { vtable, state }
    }

    /// The state of the object behind `this`.
    pub fn state(this: *mut Self) -> *mut S {
        this.cast::<u8>().wrapping_add(std::mem::offset_of!(Self, state)).cast()
    }

    /// Moves a new object onto the game heap, so that C++ `delete` can free it through slot 0.
    ///
    /// The destructor in the vtable must free with [`RustObject::destroy`] and the same heap.
    ///
    /// # Safety
    /// The heap's function pointers must be valid `operator new` and `operator delete` entry points.
    pub unsafe fn allocate_in(heap: &MsvcHeap, vtable: &'static V, state: S) -> Result<*mut Self, MsvcError> {
        let () = Self::FITS_HEAP_ALIGNMENT;
        let bytes = size_of::<Self>();
        // SAFETY: the caller guarantees the heap is valid.
        let ptr = unsafe { heap.allocate(bytes) }.cast::<Self>();
        if ptr.is_null() {
            return Err(MsvcError::AllocationFailed { bytes });
        }
        // SAFETY: `ptr` points to `size_of::<Self>()` writable bytes, aligned to 16 by the allocator.
        unsafe { ptr.write(Self::new(vtable, state)) };
        Ok(ptr)
    }

    /// The body of a scalar deleting destructor: drops the state and, when `flags` bit 0 is set, frees the
    /// object's memory through `heap`. Returns `this`.
    ///
    /// # Safety
    /// `this` must point to a live object made by [`RustObject::allocate_in`] with this heap, and must not be
    /// used again after this call.
    pub unsafe fn destroy(this: *mut Self, flags: u32, heap: &MsvcHeap) -> *mut c_void {
        // SAFETY: the caller guarantees `this` is live and not used again, so dropping the state once is sound.
        unsafe { std::ptr::drop_in_place(Self::state(this)) };
        if flags & 1 != 0 {
            // SAFETY: the caller guarantees the block came from `allocate_in` with this heap, for this size.
            unsafe { heap.deallocate(this.cast(), size_of::<Self>()) };
        }
        this.cast()
    }
}

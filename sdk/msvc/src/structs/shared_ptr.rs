use super::ref_count::RefCountBase;
use std::sync::atomic::Ordering;

/// Layout mirror of the MSVC `std::shared_ptr<T>`: element pointer, then control block pointer.
///
/// It has no `Drop`: a copy of a game object must be released on purpose with `release`.
#[repr(C)]
pub struct SharedPtr<T> {
    ptr: *mut T,
    rep: *mut RefCountBase,
}

impl<T> SharedPtr<T> {
    /// The element, or `None` for an empty pointer.
    pub fn get(&self) -> Option<&T> {
        // SAFETY: a non-null element pointer stays valid while this pointer holds a use.
        unsafe { self.ptr.as_ref() }
    }

    /// `use_count()`; 0 for an empty pointer.
    pub fn use_count(&self) -> i32 {
        if self.rep.is_null() {
            return 0;
        }
        // SAFETY: a non-null control block stays valid while this pointer holds a use.
        unsafe { (*self.rep).uses.load(Ordering::Relaxed) }
    }

    /// Copies the pointer and takes one more use, as the `shared_ptr` copy constructor does.
    ///
    /// # Safety
    /// This pointer must be empty or hold a live use.
    pub unsafe fn clone_ref(&self) -> SharedPtr<T> {
        if !self.rep.is_null() {
            // SAFETY: the caller guarantees the control block is live.
            unsafe { (*self.rep).incref() };
        }
        SharedPtr { ptr: self.ptr, rep: self.rep }
    }

    /// Drops the use this pointer holds, as the `shared_ptr` destructor does, and leaves it empty.
    ///
    /// # Safety
    /// This pointer must be empty or hold a live use.
    pub unsafe fn release(&mut self) {
        let rep = self.rep;
        self.ptr = std::ptr::null_mut();
        self.rep = std::ptr::null_mut();
        if !rep.is_null() {
            // SAFETY: the caller guarantees `rep` is live and this pointer owned one use.
            unsafe { RefCountBase::decref(rep) };
        }
    }
}

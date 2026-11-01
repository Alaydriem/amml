use std::ffi::c_void;
use std::sync::atomic::{AtomicI32, Ordering};

/// Layout mirror of the MSVC `_Ref_count_base` vtable. The slots follow the declaration order in `<memory>`.
#[repr(C)]
pub struct RefCountVtable {
    pub destroy: unsafe extern "C" fn(*mut RefCountBase),
    pub delete_this: unsafe extern "C" fn(*mut RefCountBase),
    pub destructor: unsafe extern "C" fn(*mut RefCountBase, u32) -> *mut c_void,
    pub get_deleter: unsafe extern "C" fn(*const RefCountBase, *const c_void) -> *mut c_void,
}

/// Layout mirror of the MSVC `_Ref_count_base`: vtable pointer, use count, weak count.
#[repr(C)]
pub struct RefCountBase {
    pub vtable: *const RefCountVtable,
    pub uses: AtomicI32,
    pub weaks: AtomicI32,
}

impl RefCountBase {
    /// `_Incref`. The STL increments with a relaxed atomic add, because taking a new reference needs no ordering.
    pub fn incref(&self) {
        self.uses.fetch_add(1, Ordering::Relaxed);
    }

    /// `_Decref`: drop one use, destroy the value at zero, then drop the weak count the uses held.
    ///
    /// # Safety
    /// `this` must point to a live control block, and the caller must own one use.
    pub unsafe fn decref(this: *mut RefCountBase) {
        // SAFETY: the caller guarantees `this` is live.
        let previous = unsafe { (*this).uses.fetch_sub(1, Ordering::AcqRel) };
        if previous == 1 {
            // SAFETY: `this` is live and its vtable was set by the C++ constructor.
            unsafe { ((*(*this).vtable).destroy)(this) };
            // SAFETY: the uses owned one weak reference, which this call drops.
            unsafe { Self::decwref(this) };
        }
    }

    /// `_Decwref`: drop one weak reference and free the control block at zero.
    ///
    /// # Safety
    /// `this` must point to a live control block, and the caller must own one weak reference.
    pub unsafe fn decwref(this: *mut RefCountBase) {
        // SAFETY: the caller guarantees `this` is live.
        let previous = unsafe { (*this).weaks.fetch_sub(1, Ordering::AcqRel) };
        if previous == 1 {
            // SAFETY: no reference remains, so the block frees itself exactly once.
            unsafe { ((*(*this).vtable).delete_this)(this) };
        }
    }
}

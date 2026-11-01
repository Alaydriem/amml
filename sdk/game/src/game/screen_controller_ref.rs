use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{ContainerScreenController, RawObject};
use crate::{EntryId, GameEntries};

/// A `ScreenController*` of the game, as a `ScreenView` holds it. Client main thread; the reads are not
/// thread-checked.
pub struct ScreenControllerRef<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ScreenControllerRef<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ScreenController` of the game build `entries` was resolved for, and must stay
    /// live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The address of the controller, to compare with a controller seen earlier.
    pub fn address(&self) -> usize {
        self.ptr.as_ptr() as usize
    }

    /// Whether the controller is a container screen, through `_isContainerScreen`. `None` when the entry is off.
    #[allow(unsafe_code)]
    pub fn is_container_screen(&self) -> Option<bool> {
        let slot = self.entries.get(EntryId::SCREEN_CONTROLLER_IS_CONTAINER_SCREEN)?;
        // SAFETY: `from_raw` guarantees a live controller, whose first qword is its vtable, and the entry is the slot
        // of `_isContainerScreen` in it.
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `bool ScreenController::_isContainerScreen() const`.
        Some(unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) })
    }

    /// The same controller as a container screen controller; `None` when it is not one or the test is off.
    #[allow(unsafe_code)]
    pub fn container(&self) -> Option<ContainerScreenController<'a>> {
        if !self.is_container_screen()? {
            return None;
        }
        // SAFETY: a controller whose `_isContainerScreen` is true is a ContainerScreenController.
        unsafe { ContainerScreenController::from_raw(self.ptr.as_ptr(), self.entries) }
    }
}

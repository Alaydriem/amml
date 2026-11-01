use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{RawObject, ScreenControllerRef, ScreenViewLayout};
use crate::{EntryId, GameEntries};

/// A `ScreenView*` of the game: the view that `ScreenView::render` draws.
///
/// Every read needs the `ScreenView.mPointerLocationPrevious` entry, which anchors the mirror: with it off, every
/// read gives `None`. Client main thread; the reads are not thread-checked.
pub struct ScreenView<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ScreenView<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ScreenView` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The pointer position of the last frame (`mPointerLocationPrevious`), in GUI units. `None` when the entry is
    /// off.
    #[allow(unsafe_code)]
    pub fn pointer(&self) -> Option<[f32; 2]> {
        let offset = self.entries.get(EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS)?;
        // SAFETY: `from_raw` guarantees a live ScreenView, and the entry is the offset of the vec2 in it.
        unsafe { RawObject::read::<[f32; 2]>(self.ptr.as_ptr(), offset) }
    }

    /// The size of the view (`mSize`), in GUI units. `None` when the entry is off.
    #[allow(unsafe_code)]
    pub fn size(&self) -> Option<[f32; 2]> {
        self.entries.get(EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS)?;
        // SAFETY: `from_raw` guarantees a live ScreenView, which the prefix mirror fits.
        Some(unsafe { self.ptr.as_ptr().cast::<ScreenViewLayout>().read() }.size)
    }

    /// The controller of the view (`mController`). `None` when the entry is off or the view has none.
    #[allow(unsafe_code)]
    pub fn controller(&self) -> Option<ScreenControllerRef<'a>> {
        self.entries.get(EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS)?;
        // SAFETY: `from_raw` guarantees a live ScreenView, which the prefix mirror fits.
        let controller = unsafe { self.ptr.as_ptr().cast::<ScreenViewLayout>().read() }.controller;
        // SAFETY: the shared_ptr in a live view holds null or a live controller.
        unsafe { ScreenControllerRef::from_raw(controller, self.entries) }
    }
}

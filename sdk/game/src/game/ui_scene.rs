use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{RawObject, ScreenView};
use crate::{EntryId, GameEntries};

/// A `UIScene*` of the game: the scene that `UIScene::handleDirection` receives the wheel steps on, and whose
/// `ScreenView` `ScreenView::render` draws. Client main thread; the reads are not thread-checked.
pub struct UiScene<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> UiScene<'a> {
    /// The feature of the UI wheel hook: `UIScene::handleDirection` and the scene's `ScreenView` offset.
    pub const FEATURE: &'static str = "game.ui_scroll";

    /// The entries of `FEATURE`.
    pub const ENTRIES: [EntryId; 2] = [EntryId::UI_SCENE_HANDLE_DIRECTION, EntryId::UI_SCENE_SCREEN_VIEW];

    /// The `DirectionId` of a wheel step (`DirectionId::Scroll`).
    pub const DIRECTION_SCROLL: u8 = 2;

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `UIScene` of the game build `entries` was resolved for, and must stay live while
    /// the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The view of the scene (`UIScene.mScreenView`). `None` when the entry is off or the scene has no view.
    #[allow(unsafe_code)]
    pub fn screen_view(&self) -> Option<ScreenView<'a>> {
        let offset = self.entries.get(EntryId::UI_SCENE_SCREEN_VIEW)?;
        // SAFETY: `from_raw` guarantees a live UIScene, and the entry is the offset of its ScreenView pointer.
        let view = unsafe { RawObject::pointer(self.ptr.as_ptr(), offset) }?;
        // SAFETY: the pointer of a live scene is a live ScreenView of the same build.
        unsafe { ScreenView::from_raw(view, self.entries) }
    }
}

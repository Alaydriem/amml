use std::ffi::c_void;

use amml_msvc::SharedPtr;

use crate::FontRepositoryLayout;

/// Layout mirror of the game's `FontHandle` (0x50 bytes), which `ClientInstance::getFontHandle` returns by value.
#[repr(C)]
pub struct FontHandleLayout {
    _base: [u8; 0x18],                       // 0x00 EnableNonOwnerReferences: vtable, control block
    _repository_control: [u8; 0x10],         // 0x18 mFontRepository: control block
    repository: *const FontRepositoryLayout, // 0x28 mFontRepository: the pointer
    default_font: SharedPtr<c_void>,         // 0x30 mDefaultFont
    font_id: u64,                            // 0x40 mFontId
    _is_dummy_handle: bool,                  // 0x48 mIsDummyHandle
}

impl FontHandleLayout {
    /// The `Font` of the handle: the font at index `mFontId` of the repository when the id is in range, otherwise
    /// the default font. `None` when the picked pointer is null.
    ///
    /// # Safety
    /// The handle must have been filled by the game, and its repository must be live.
    #[allow(unsafe_code)]
    pub unsafe fn font(&self) -> Option<*mut c_void> {
        // SAFETY: the caller guarantees the repository pointer is null or a live repository.
        let repository = unsafe { self.repository.as_ref() };
        let picked = repository
            .and_then(|repository| usize::try_from(self.font_id).ok().and_then(|index| repository.fonts().get(index)));
        let font = picked.unwrap_or(&self.default_font);
        font.get().map(|font| std::ptr::from_ref(font).cast_mut())
    }
}

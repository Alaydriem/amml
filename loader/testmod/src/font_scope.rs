use std::ffi::c_void;
use std::mem::MaybeUninit;

use amml_game::{GameEntries, RawObject};

use crate::{FontHandleLayout, PeekEntries};

/// The default `FontHandle` of the client, held while text is queued. Dropping it runs the game's destructor, so it
/// must live until `flush_text` has run. Client main thread.
pub struct FontScope {
    handle: Box<MaybeUninit<FontHandleLayout>>,
    destroy: usize,
}

impl FontScope {
    /// Asks `client` for its font handle. `None` when an entry is off, the size entry is not the mirror size, or a
    /// pointer is null.
    ///
    /// # Safety
    /// `client` must be a live `ClientInstance` of the game build `entries` was resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn open(entries: &GameEntries, client: *mut c_void) -> Option<Self> {
        let slot = entries.get(PeekEntries::GET_FONT_HANDLE)?;
        let destroy = usize::try_from(entries.get(PeekEntries::FONT_HANDLE_DTOR)?).ok()?;
        if destroy == 0
            || client.is_null()
            || entries.get(PeekEntries::FONT_HANDLE_SIZE) != Some(size_of::<FontHandleLayout>() as u64)
        {
            return None;
        }
        // SAFETY: the caller guarantees a live client, whose first qword is its vtable, and the entry is the slot of
        // `getFontHandle` in it.
        let function = unsafe { RawObject::slot(client, slot) }?;
        // SAFETY: the slot holds `FontHandle ClientInstance::getFontHandle() const`: this in rcx, the hidden return
        // buffer in rdx.
        let get_handle: unsafe extern "C" fn(*mut c_void, *mut FontHandleLayout) -> *mut c_void =
            unsafe { std::mem::transmute(function) };
        let mut handle = Box::new(MaybeUninit::<FontHandleLayout>::zeroed());
        // SAFETY: the buffer is a FontHandle, whose size was checked against the build, and the client is live.
        unsafe { get_handle(client, handle.as_mut_ptr()) };
        Some(Self { handle, destroy })
    }

    /// The `Font*` of the handle; valid while `self` lives. `None` when the picked pointer is null.
    #[allow(unsafe_code)]
    pub fn font(&self) -> Option<*mut c_void> {
        // SAFETY: `open` had the game fill the handle just now, and its repository is the game's.
        unsafe { (*self.handle.as_ptr()).font() }
    }
}

impl Drop for FontScope {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: the entry is the address of the non-deleting `FontHandle::~FontHandle()`, which takes this in rcx.
        let destroy: unsafe extern "C" fn(*mut FontHandleLayout) =
            unsafe { std::mem::transmute(self.destroy as *const c_void) };
        // SAFETY: the handle was filled by the game and is destroyed exactly once.
        unsafe { destroy(self.handle.as_mut_ptr()) };
    }
}

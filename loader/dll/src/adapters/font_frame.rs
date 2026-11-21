use std::ffi::c_void;

use crate::TextCalls;

/// The `FontHandle` and the texts of one `ScreenRender` dispatch: the handle is a buffer of the game's size that
/// `ClientInstance::getFontHandle` filled, destroyed by the game's destructor when the frame drops. `serial` is the
/// serial of the dispatch it was opened for; `font` is the `Font*` the handle picked. The game keeps the view of a
/// text it was given and not its bytes, so `texts` holds the copies until the frame drops, after the flush.
pub struct FontFrame {
    pub serial: u64,
    pub font: usize,
    pub texts: Vec<Box<[u8]>>,
    // 8-byte aligned words, as the handle needs
    handle: Box<[u64; TextCalls::HANDLE_SIZE / size_of::<u64>()]>,
    dtor: usize,
}

impl FontFrame {
    /// A frame over `handle`, which the game filled; `dtor` is the destructor of the handle.
    pub fn new(serial: u64, handle: Box<[u64; TextCalls::HANDLE_SIZE / size_of::<u64>()]>, dtor: usize) -> Self {
        Self { serial, font: 0, texts: Vec::new(), handle, dtor }
    }

    /// The address of the handle object.
    pub fn address(&self) -> usize {
        self.handle.as_ptr().addr()
    }

    /// The address a `getFontHandle` call writes the handle to.
    pub fn buffer(&mut self) -> *mut c_void {
        self.handle.as_mut_ptr().cast()
    }
}

impl Drop for FontFrame {
    fn drop(&mut self) {
        // SAFETY: the address is the non-deleting destructor of `FontHandle`, which takes this in rcx.
        let destroy = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(*mut c_void)>(self.dtor) };
        // SAFETY: the buffer holds the handle the game filled, destroyed exactly once.
        unsafe { destroy(self.handle.as_mut_ptr().cast()) };
    }
}

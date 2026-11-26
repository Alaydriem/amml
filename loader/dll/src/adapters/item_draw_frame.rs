use std::ffi::c_void;

/// The `BaseActorRenderContext` the item draws of one `ScreenRender` dispatch share: a buffer of the game's size,
/// built by the game's constructor and destroyed by its destructor when the frame drops. `serial` is the serial of the
/// dispatch it was built for; `holder` is the local player (0 for none), whose animation frame an item takes.
pub struct ItemDrawFrame {
    pub serial: u64,
    pub renderer: usize,
    pub holder: usize,
    // 16-byte aligned words, so the buffer meets the object's alignment
    buffer: Vec<u128>,
    dtor: usize,
}

impl ItemDrawFrame {
    /// A frame over `buffer`, which the game's constructor filled; `dtor` is its destructor.
    pub fn new(serial: u64, renderer: usize, holder: usize, buffer: Vec<u128>, dtor: usize) -> Self {
        Self { serial, renderer, holder, buffer, dtor }
    }

    /// The address of the context object.
    pub fn address(&self) -> usize {
        self.buffer.as_ptr().addr()
    }
}

impl Drop for ItemDrawFrame {
    fn drop(&mut self) {
        // SAFETY: the address is the complete destructor of `BaseActorRenderContext`, which takes this in rcx.
        let destroy = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(*mut c_void)>(self.dtor) };
        // SAFETY: the buffer holds the object the constructor built, destroyed exactly once.
        unsafe { destroy(self.buffer.as_mut_ptr().cast()) };
    }
}

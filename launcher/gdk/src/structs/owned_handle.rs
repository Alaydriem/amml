use windows::Win32::Foundation::{CloseHandle, HANDLE};

// A Win32 handle that closes itself.
pub struct OwnedHandle(HANDLE);

impl OwnedHandle {
    pub fn new(handle: HANDLE) -> OwnedHandle {
        OwnedHandle(handle)
    }

    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful Win32 call and `OwnedHandle` is its only owner, so it is closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

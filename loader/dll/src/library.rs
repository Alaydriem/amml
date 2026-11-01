use std::ffi::{CString, c_void};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LoadLibraryExW,
};

/// A DLL mapped into the process. It has no `Drop`: a mod DLL stays mapped until the process ends,
/// because the game may hold objects whose vtables live in it.
pub struct Library {
    handle: usize,
}

impl Library {
    /// Maps the DLL at `path` (made absolute); DLLs beside it and in the default system folders are
    /// found as its dependencies.
    ///
    /// Not for use under the OS loader lock.
    pub fn open(path: &Path) -> Result<Self, std::io::Error> {
        let path = std::path::absolute(path)?;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 path that lives for the call; the file handle
        // argument must be null.
        let handle = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            )
        };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self { handle: handle as usize })
    }

    /// The export `name` as a function pointer of type `F`, or `None` when the DLL has no such export.
    ///
    /// # Safety
    /// `F` must be an `extern` function pointer type that matches the export.
    pub unsafe fn symbol<F: Copy>(&self, name: &str) -> Option<F> {
        let name = CString::new(name).ok()?;
        if std::mem::size_of::<F>() != std::mem::size_of::<*mut c_void>() {
            return None;
        }
        // SAFETY: `handle` is a module handle from `open` and `name` is NUL-terminated.
        let address = unsafe { GetProcAddress(self.handle as *mut c_void, name.as_ptr().cast()) }?;
        // SAFETY: `F` is a function pointer type of pointer size, and the caller guarantees it
        // matches the export.
        Some(unsafe { std::mem::transmute_copy::<_, F>(&address) })
    }
}

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use amml_runtime::ImageLayout;
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};

/// Facts about the running game process.
pub struct HostProcess;

impl HostProcess {
    const MAX_PATH_UNITS: usize = 32768;
    const HEADER_BYTES: usize = 0x1000;

    /// The base address of the game module (the running exe).
    pub fn module_base() -> u64 {
        // SAFETY: a null module name asks for the module of the calling process's exe.
        let handle = unsafe { GetModuleHandleW(std::ptr::null()) };
        handle as usize as u64
    }

    /// The size and executable sections of the game image mapped at `module_base`, read from its PE
    /// headers. An unreadable header gives an empty layout, which accepts no address.
    pub fn image_layout(module_base: u64) -> ImageLayout {
        if module_base == 0 {
            return ImageLayout::empty();
        }
        // SAFETY: `module_base` is the base of the running exe's mapped image, whose first page holds
        // its headers and is always readable.
        let headers = unsafe { std::slice::from_raw_parts(module_base as usize as *const u8, Self::HEADER_BYTES) };
        ImageLayout::from_headers(headers).unwrap_or_else(ImageLayout::empty)
    }

    /// The file path of the running exe.
    pub fn exe_path() -> Option<PathBuf> {
        let mut capacity = 512usize;
        while capacity <= Self::MAX_PATH_UNITS {
            let mut buffer = vec![0u16; capacity];
            // SAFETY: `buffer` is writable for `capacity` UTF-16 units, which fits in `u32`.
            let written = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buffer.as_mut_ptr(), capacity as u32) };
            let written = written as usize;
            if written == 0 {
                return None;
            }
            if written < capacity {
                return Some(PathBuf::from(OsString::from_wide(&buffer[..written])));
            }
            capacity *= 2;
        }
        None
    }
}

use super::WindowProbe;
use crate::WindowSearch;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};
use windows::core::BOOL;

// Looks for a visible top-level window owned by a process.
#[derive(Default)]
pub struct WindowsWindowProbe;

impl WindowsWindowProbe {
    pub fn new() -> WindowsWindowProbe {
        WindowsWindowProbe
    }

    // the callback of `EnumWindows`; it returns FALSE to stop at the first match
    #[allow(unsafe_code)]
    unsafe extern "system" fn visit(window: HWND, param: LPARAM) -> BOOL {
        // SAFETY: `param` is the address of the `WindowSearch` that `has_window` keeps alive for the whole
        // `EnumWindows` call, and the callback runs on the same thread, so no other reference exists.
        let search = unsafe { &mut *(param.0 as *mut WindowSearch) };
        let mut owner = 0u32;
        // SAFETY: `window` is the handle `EnumWindows` passes in and `owner` outlives the call.
        unsafe { GetWindowThreadProcessId(window, Some(&mut owner)) };
        // SAFETY: `window` is the handle `EnumWindows` passes in.
        if owner == search.pid && unsafe { IsWindowVisible(window) }.as_bool() {
            search.found = true;
            return BOOL(0);
        }
        BOOL(1)
    }
}

impl WindowProbe for WindowsWindowProbe {
    #[allow(unsafe_code)]
    fn has_window(&self, pid: u32) -> bool {
        let mut search = WindowSearch { pid, found: false };
        // SAFETY: `visit` runs synchronously inside this call and only reads and writes `search`, which lives until
        // the call returns. A stopped enumeration returns an error that carries no information here.
        let _ = unsafe { EnumWindows(Some(Self::visit), LPARAM(&mut search as *mut WindowSearch as isize)) };
        search.found
    }
}

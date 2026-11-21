use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW, WM_CLOSE};

/// One message the game's stand-in procedure saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
}

static SEEN: Mutex<Vec<Seen>> = Mutex::new(Vec::new());

/// A recording window procedure that plays the game's own procedure under the loader's subclass. It answers
/// `WM_CLOSE` with 0 and keeps the window, so a test can send it again.
pub struct MessageLog;

impl MessageLog {
    /// Makes the recorder the window's procedure, under whatever the test installs next.
    pub fn install(hwnd: HWND) {
        SEEN.lock().unwrap().clear();
        // SAFETY: `record` has the signature of a window procedure and lives for the process.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, Self::record as *const () as isize) };
    }

    /// What reached the recorder, oldest first.
    pub fn seen() -> Vec<Seen> {
        SEEN.lock().unwrap().clone()
    }

    /// What reached the recorder of `message`, oldest first.
    pub fn seen_of(message: u32) -> Vec<Seen> {
        Self::seen().into_iter().filter(|seen| seen.message == message).collect()
    }

    pub fn clear() {
        SEEN.lock().unwrap().clear();
    }

    unsafe extern "system" fn record(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        SEEN.lock().unwrap().push(Seen { message, wparam: wparam.0, lparam: lparam.0 });
        if message == WM_CLOSE {
            return LRESULT(0);
        }
        // SAFETY: the arguments of a live message.
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}

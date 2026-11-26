use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, DefWindowProcW, WNDPROC};

use super::{SurrogateJoin, WindowProcDepth};

/// What the window procedure reads on every message: the procedure the window had before the loader's, whether
/// routing is on, and the surrogate join of `WM_CHAR`. Atomics only, so the procedure takes no lock.
pub struct RouterState {
    // The procedure the window had before the loader's, as an address; 0 when none.
    original: AtomicUsize,
    // A panic in the loader's routing turned it off for the rest of the process.
    disabled: AtomicBool,
    thread_logged: AtomicBool,
    surrogates: SurrogateJoin,
}

impl Default for RouterState {
    fn default() -> Self {
        Self::new()
    }
}

impl RouterState {
    pub const fn new() -> Self {
        Self {
            original: AtomicUsize::new(0),
            disabled: AtomicBool::new(false),
            thread_logged: AtomicBool::new(false),
            surrogates: SurrogateJoin::new(),
        }
    }

    pub fn set_original(&self, address: usize) {
        self.original.store(address, Ordering::SeqCst);
    }

    /// Routing has not been turned off by a panic.
    pub fn active(&self) -> bool {
        !self.disabled.load(Ordering::SeqCst)
    }

    /// Turns routing off after a panic. True the first time only, for the one log line.
    pub fn disable(&self) -> bool {
        !self.disabled.swap(true, Ordering::SeqCst)
    }

    /// True the first time only.
    pub fn first_call(&self) -> bool {
        !self.thread_logged.swap(true, Ordering::Relaxed)
    }

    pub fn surrogates(&self) -> &SurrogateJoin {
        &self.surrogates
    }

    /// Runs the window's own procedure (or `DefWindowProcW` when there is none) with one message.
    pub fn pass_on(&self, hwnd: usize, message: u32, wparam: usize, lparam: isize) -> LRESULT {
        let window = HWND(std::ptr::with_exposed_provenance_mut::<c_void>(hwnd));
        let address = self.original.load(Ordering::SeqCst);
        // SAFETY: the address is 0 or the procedure `SetWindowLongPtrW` gave back for the window; 0 becomes `None`,
        // as for any `Option` of a function pointer.
        let original = unsafe { std::mem::transmute::<usize, WNDPROC>(address) };
        match original {
            // SAFETY: the window's procedure from before the loader's, called with the message the loader got.
            Some(_) => WindowProcDepth::run(|| unsafe {
                CallWindowProcW(original, window, message, WPARAM(wparam), LPARAM(lparam))
            }),
            // SAFETY: the arguments of a live message.
            None => unsafe { DefWindowProcW(window, message, WPARAM(wparam), LPARAM(lparam)) },
        }
    }
}

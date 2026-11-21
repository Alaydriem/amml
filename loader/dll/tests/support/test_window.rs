use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, RegisterClassExW,
    SendMessageW, WINDOW_EX_STYLE, WNDCLASSEXW, WS_POPUP,
};
use windows::core::{PCWSTR, w};

/// A hidden popup window of a test, destroyed on drop. Its client area is its whole size.
pub struct TestWindow {
    pub hwnd: HWND,
}

impl TestWindow {
    const CLASS: PCWSTR = w!("AmmlLoaderTestWindow");

    pub fn new(width: u32, height: u32) -> Self {
        // SAFETY: the module of the process stays loaded.
        let instance = HINSTANCE(unsafe { GetModuleHandleW(None) }.unwrap().0);
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(Self::window_proc),
            hInstance: instance,
            lpszClassName: Self::CLASS,
            ..Default::default()
        };
        // SAFETY: `class` lives for the call; a second registration fails and the class stays usable.
        unsafe { RegisterClassExW(&class) };
        // SAFETY: the class is registered and the strings are static.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                Self::CLASS,
                w!(""),
                WS_POPUP,
                0,
                0,
                width as i32,
                height as i32,
                None,
                None,
                Some(instance),
                None,
            )
        }
        .unwrap();
        Self { hwnd }
    }

    /// The handle as the loader takes it.
    pub fn address(&self) -> usize {
        self.hwnd.0 as usize
    }

    /// Sends one message to the window on this thread and gives its result.
    pub fn send(&self, message: u32, wparam: usize, lparam: isize) -> isize {
        // SAFETY: the window is live and belongs to this thread.
        unsafe { SendMessageW(self.hwnd, message, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }.0
    }

    /// Runs every message posted to this thread's windows, also the ones the runs post.
    pub fn pump(&self) {
        let mut message = MSG::default();
        // SAFETY: `message` is a writable local, and the messages are this thread's.
        while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
            // SAFETY: a message just taken from the queue.
            unsafe { DispatchMessageW(&message) };
        }
    }

    /// The `lParam` of a mouse message at client point `x`, `y`.
    pub fn point(x: i16, y: i16) -> isize {
        ((i32::from(y) << 16) | i32::from(x as u16)) as isize
    }

    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // SAFETY: the arguments of a live message.
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        // SAFETY: made by this value on this thread.
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

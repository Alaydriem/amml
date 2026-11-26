use std::sync::{Mutex, MutexGuard};

use amml_runtime::{ClipBook, ClipKind, ClipStep};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    ClipCursor, GetClientRect, GetForegroundWindow, IsIconic, SIZE_MINIMIZED, WM_ACTIVATE, WM_ACTIVATEAPP,
    WM_KILLFOCUS, WM_SETFOCUS, WM_SIZE, WM_WINDOWPOSCHANGED,
};

use crate::WindowService;

/// The system cursor clip of the game window, set and given back for the requests of mods (`ClipBook` decides).
///
/// The clip is a 2 x 2 box at the client centre. The book and the Win32 call stay under one lock, so the clip the
/// system holds always follows the order of the changes. A call may come from the window thread (focus, size and
/// move messages) or from the thread of a mod; `ClipCursor` has no thread rule.
pub struct CursorClipAdapter {
    book: Mutex<ClipBook>,
}

impl Default for CursorClipAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorClipAdapter {
    pub fn new() -> Self {
        Self { book: Mutex::new(ClipBook::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, ClipBook> {
        self.book.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sets the request of `owner` for the window `hwnd`, or drops it with `None`. The window state is read first,
    /// so a request made while the window is in front is applied at once.
    ///
    /// A `Some` request is added only when `allowed` says so under the lock, so a request that comes after
    /// `end_owner` of the same mod is refused and leaves no clip. Gives false for a refused request.
    pub fn request(&self, hwnd: usize, owner: usize, clip: Option<ClipKind>, allowed: impl FnOnce() -> bool) -> bool {
        let mut book = self.lock();
        if clip.is_some() && !allowed() {
            return false;
        }
        let step = book.set_active(Self::window_active(hwnd));
        Self::apply(hwnd, step);
        let step = book.request(owner, clip);
        Self::apply(hwnd, step);
        true
    }

    /// Drops the request of `owner`; `hwnd` is the attached window, if any.
    pub fn end_owner(&self, hwnd: Option<usize>, owner: usize) {
        let step = self.lock().end_owner(owner);
        Self::apply_to(hwnd, step);
    }

    /// One message of the window `hwnd`, from the loader's procedure: a focus, size or move message changes the
    /// clip.
    pub fn message(&self, hwnd: usize, message: u32, wparam: usize) {
        if !matches!(message, WM_ACTIVATE | WM_ACTIVATEAPP | WM_SETFOCUS | WM_KILLFOCUS | WM_SIZE | WM_WINDOWPOSCHANGED)
        {
            return;
        }
        let mut book = self.lock();
        let step = match message {
            // The low word of `wParam` is `WA_INACTIVE` (0) when the window is deactivated.
            WM_ACTIVATE => book.set_active(wparam & 0xFFFF != 0 && Self::window_active(hwnd)),
            WM_ACTIVATEAPP => book.set_active(wparam != 0 && Self::window_active(hwnd)),
            WM_SETFOCUS | WM_KILLFOCUS => book.set_active(Self::window_active(hwnd)),
            WM_SIZE if wparam == SIZE_MINIMIZED as usize => book.set_active(false),
            WM_SIZE | WM_WINDOWPOSCHANGED => Self::resync(&mut book, hwnd),
            _ => ClipStep::Keep,
        };
        Self::apply(hwnd, step);
    }

    // Reads the window state, then applies the wanted clip at the current centre. A minimised window is not
    // active, so no clip is set at its off-screen place and a clip that is set is given back.
    fn resync(book: &mut ClipBook, hwnd: usize) -> ClipStep {
        match book.set_active(Self::window_active(hwnd)) {
            ClipStep::Release => ClipStep::Release,
            _ => book.moved(),
        }
    }

    /// The attached window is destroyed: the clip is given back and the requests stay for the next window.
    pub fn detach(&self, hwnd: usize) {
        let step = self.lock().window_gone();
        Self::apply(hwnd, step);
    }

    // Whether the window is the foreground window and not minimised.
    fn window_active(hwnd: usize) -> bool {
        let window = WindowService::handle(hwnd);
        // SAFETY: no input.
        let foreground = unsafe { GetForegroundWindow() } == window;
        // SAFETY: a handle that is not a window gives false.
        foreground && !unsafe { IsIconic(window) }.as_bool()
    }

    fn apply_to(hwnd: Option<usize>, step: ClipStep) {
        match (hwnd, step) {
            (Some(hwnd), step) => Self::apply(hwnd, step),
            (None, ClipStep::Release) => Self::release(),
            (None, _) => {}
        }
    }

    fn apply(hwnd: usize, step: ClipStep) {
        match step {
            ClipStep::Keep => {}
            ClipStep::Release => Self::release(),
            ClipStep::Apply(ClipKind::Centre) => {
                let window = WindowService::handle(hwnd);
                let mut client = RECT::default();
                // SAFETY: `client` is a writable local; a handle that is not a window makes the call fail.
                if unsafe { GetClientRect(window, &mut client) }.is_err() {
                    return;
                }
                let mut centre = POINT { x: (client.right - client.left) / 2, y: (client.bottom - client.top) / 2 };
                // SAFETY: `centre` is a writable local; a handle that is not a window makes the call fail.
                if !unsafe { ClientToScreen(window, &mut centre) }.as_bool() {
                    return;
                }
                let clip = RECT { left: centre.x - 1, top: centre.y - 1, right: centre.x + 1, bottom: centre.y + 1 };
                // SAFETY: `clip` is a live rectangle for the call.
                if let Err(error) = unsafe { ClipCursor(Some(&raw const clip)) } {
                    curia::warn!("cursor clip not set", { error: error.to_string() });
                }
            }
        }
    }

    fn release() {
        // SAFETY: no rectangle gives the clip back.
        if let Err(error) = unsafe { ClipCursor(None) } {
            curia::warn!("cursor clip not released", { error: error.to_string() });
        }
    }
}

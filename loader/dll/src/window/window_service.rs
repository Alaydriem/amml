use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use amml_abi::{AmmlPoint, AmmlSize, AmmlStatus};
use amml_data::{AdapterCatalog, NeedKind};
use amml_runtime::{BufferMap, ClipKind, FocusHold, PointerPairs};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GUITHREADINFO, GWLP_WNDPROC, GetClientRect, GetCursorPos, GetForegroundWindow, GetGUIThreadInfo, GetPropW,
    GetWindowLongPtrW, GetWindowThreadProcessId, IsChild, IsWindow, PostMessageW, RemovePropW, SetPropW,
    SetWindowLongPtrW,
};
use windows::core::{PCWSTR, w};

use super::{RouterState, SizeMessage, SizeRequests, WindowProc};
use crate::{AreaBook, CursorClipAdapter, EventHub, ShutdownLatch};

/// The game window as the loader holds it: the one subclass, the reads and requests of the `window` table, and the
/// state the window procedure keeps between messages (pointer pairs, held focus losses, size messages the loader
/// posted, the client to back buffer map).
///
/// The window is attached once, by the overlay bind, and keeps the loader's procedure for the process: a later bind
/// on the same window does not subclass it again, also when another module subclassed it after the loader (its
/// procedure then calls the loader's, which must stay where it is in the chain). The loader never puts a procedure
/// back. A second window is ignored unless the attached one no longer exists; the window procedure tells the service
/// when the attached window is destroyed (`detach`). Every read and request gives `NOT_AVAILABLE` while its area is
/// not offered and `NOT_READY` before a window is attached and after it was destroyed.
pub struct WindowService {
    events: Arc<EventHub>,
    areas: Arc<AreaBook>,
    shutdown: Arc<ShutdownLatch>,
    // The attached window, held across `attach`; 0 when none.
    window: Mutex<usize>,
    // The attached window for lock-free reads; 0 when none.
    hwnd: AtomicUsize,
    // The attached window carries the `MARK` property.
    marked: AtomicBool,
    // A procedure added after the loader's was logged.
    later_logged: AtomicBool,
    router: RouterState,
    buffer: BufferMap,
    pairs: PointerPairs,
    focus: FocusHold,
    sizes: SizeRequests,
    clip: CursorClipAdapter,
}

impl WindowService {
    /// The area of the reads and `replay_focus_loss`.
    pub const READ_AREA: &'static str = "window.read";
    /// The area of `request_size`.
    pub const REQUEST_SIZE_AREA: &'static str = "window.request_size";
    /// The area of `clip_cursor`.
    pub const CURSOR_CLIP_AREA: &'static str = "window.cursor_clip";
    const AREA_VERSION: u32 = 1;
    // The largest side a `WM_SIZE` can carry.
    const MAX_SIDE: u32 = 0xFFFF;
    // The window property the loader sets on the window it subclassed; Windows drops it with the window.
    const MARK: PCWSTR = w!("AmmlLoaderSubclass");

    pub fn new(events: Arc<EventHub>, areas: Arc<AreaBook>, shutdown: Arc<ShutdownLatch>) -> Self {
        Self {
            events,
            areas,
            shutdown,
            window: Mutex::new(0),
            hwnd: AtomicUsize::new(0),
            marked: AtomicBool::new(false),
            later_logged: AtomicBool::new(false),
            router: RouterState::new(),
            buffer: BufferMap::new(),
            pairs: PointerPairs::new(),
            focus: FocusHold::new(),
            sizes: SizeRequests::new(),
            clip: CursorClipAdapter::new(),
        }
    }

    fn lock_window(&self) -> MutexGuard<'_, usize> {
        self.window.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Offers the window events and areas (shape `window` in the catalog). The overlay install calls it once its
    /// hooks are in, so a host where the window is never attached does not offer them.
    pub fn offer(&self) {
        for need in AdapterCatalog::needs() {
            if !need.shapes.iter().any(|shape| shape.name == AdapterCatalog::WINDOW_SHAPE) {
                continue;
            }
            match need.kind {
                NeedKind::Event => self.events.offer(need.id, need.version),
                NeedKind::Area => self.areas.offer(need.id, need.version),
            }
        }
    }

    /// Subclasses the window `hwnd` with the loader's procedure. True when it is attached (also when it already
    /// was). The attached window is never subclassed again: when its procedure is not the loader's, another module
    /// subclassed it after the loader, and its procedure passes messages on to the loader's (logged once). A second
    /// window is ignored and logged while the attached one exists; once it is gone, the new one is attached in its
    /// place, and what the loader kept for the old window (button pairing, held focus losses and their replay marks,
    /// size messages not run yet, a half surrogate pair) is forgotten. A handle value Windows gave again to a new
    /// window, which has neither the loader's procedure nor its mark, is subclassed again.
    pub fn attach(&self, hwnd: usize) -> bool {
        let mut attached = self.lock_window();
        if hwnd == 0 {
            curia::warn!("window not attached: no window", { hwnd: hwnd });
            return false;
        }
        let window = Self::handle(hwnd);
        // SAFETY: a handle that is not a window gives 0; the call only reads the window's procedure.
        let current = unsafe { GetWindowLongPtrW(window, GWLP_WNDPROC) };
        let ours = current as usize == WindowProc::address();
        if *attached == hwnd {
            // With no mark set at the attach, the handle cannot be told from a new window that took its value; the
            // loader keeps its place rather than risk a chain that calls itself.
            if ours || Self::has_mark(window) || !self.marked.load(Ordering::Acquire) {
                if !ours && !self.later_logged.swap(true, Ordering::AcqRel) {
                    curia::info!("another procedure subclassed the game window after the loader, the loader keeps its place", {
                        hwnd: hwnd
                    });
                }
                return true;
            }
            curia::info!("the attached window handle came back as a new window", { hwnd: hwnd });
        } else if ours || Self::has_mark(window) {
            curia::warn!("window not attached: the loader's procedure is on it already", { hwnd: hwnd });
            return false;
        } else if *attached != 0 {
            if Self::is_window(*attached) {
                curia::warn!("window not attached: a window is attached already", { attached: *attached, hwnd: hwnd });
                return false;
            }
            curia::info!("the attached window is gone, the new window takes its place", { gone: *attached, hwnd: hwnd });
        }
        if current == 0 {
            curia::warn!("window not attached: it has no window procedure", { hwnd: hwnd });
            return false;
        }
        let replaces = *attached != 0;
        // A message that arrives between the swap and the store below still finds the procedure the window had.
        self.router.set_original(current as usize);
        // SAFETY: `WindowProc::window_proc` has the signature of a window procedure and lives for the process.
        let previous = unsafe { SetWindowLongPtrW(window, GWLP_WNDPROC, WindowProc::address() as isize) };
        if previous == 0 {
            curia::warn!("window not attached: Windows refused the subclass", { hwnd: hwnd });
            return false;
        }
        self.router.set_original(previous as usize);
        if replaces {
            self.forget_window_state();
        }
        // SAFETY: the property name is a static string and the value is a plain number, not a pointer.
        let marked = unsafe { SetPropW(window, Self::MARK, Some(HANDLE(std::ptr::without_provenance_mut(1)))) };
        if marked.is_err() {
            curia::warn!("the game window could not be marked, a later attach keeps the subclass", { hwnd: hwnd });
        }
        self.marked.store(marked.is_ok(), Ordering::Release);
        self.later_logged.store(false, Ordering::Release);
        *attached = hwnd;
        self.hwnd.store(hwnd, Ordering::Release);
        self.refresh_client(hwnd);
        curia::info!("game window subclassed", { hwnd: hwnd });
        true
    }

    /// The window `hwnd` gets `WM_NCDESTROY`, from the loader's procedure: when it is the attached window, it is
    /// forgotten with what the loader kept for it, so the next attach takes a new window, also one that gets the
    /// same handle value. The procedure the message is passed on to stays set for the rest of this message.
    pub fn detach(&self, hwnd: usize) {
        let mut attached = self.lock_window();
        if *attached != hwnd || hwnd == 0 {
            return;
        }
        // SAFETY: the window is live for the message that called this; the property name is a static string.
        let _ = unsafe { RemovePropW(Self::handle(hwnd), Self::MARK) };
        *attached = 0;
        self.hwnd.store(0, Ordering::Release);
        self.marked.store(false, Ordering::Release);
        self.clip.detach(hwnd);
        self.forget_window_state();
        curia::info!("game window destroyed, the loader forgets it", { hwnd: hwnd });
    }

    /// The attached window; loader-internal, never given to a mod.
    pub fn hwnd(&self) -> Option<usize> {
        match self.hwnd.load(Ordering::Acquire) {
            0 => None,
            hwnd => Some(hwnd),
        }
    }

    /// The window procedure's routing state.
    pub fn router(&self) -> &RouterState {
        &self.router
    }

    /// The window and mouse feed pairing of button presses.
    pub fn pairs(&self) -> &PointerPairs {
        &self.pairs
    }

    /// The focus losses listeners held back.
    pub fn focus(&self) -> &FocusHold {
        &self.focus
    }

    /// The size requests of mods and the size messages the loader posted.
    pub fn sizes(&self) -> &SizeRequests {
        &self.sizes
    }

    /// The cursor clip of the window: focus, size and move messages reach it from the window procedure.
    pub fn clip(&self) -> &CursorClipAdapter {
        &self.clip
    }

    /// The client to back buffer map.
    pub fn buffer_map(&self) -> &BufferMap {
        &self.buffer
    }

    /// Sets the back buffer size of the bound swap chain (the overlay, at bind and after a resize).
    pub fn set_buffer(&self, size: AmmlSize) {
        self.buffer.set_buffer(size);
    }

    /// Reads the client size of `hwnd` into the buffer map; gives it, or `None` when Windows refused.
    pub fn refresh_client(&self, hwnd: usize) -> Option<AmmlSize> {
        let size = Self::client_rect(hwnd)?;
        self.buffer.set_client(size);
        Some(size)
    }

    /// The point `client` in back buffer coordinates; `client` itself while the buffer size is not known.
    pub fn buffer_point(&self, hwnd: usize, client: AmmlPoint) -> AmmlPoint {
        self.refresh_client(hwnd);
        self.buffer.to_buffer(client).unwrap_or(client)
    }

    pub fn client_size(&self) -> Result<AmmlSize, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        Self::client_rect(hwnd).ok_or(AmmlStatus::FAILED)
    }

    /// `NOT_READY` until the overlay set the back buffer size.
    pub fn buffer_size(&self) -> Result<AmmlSize, AmmlStatus> {
        self.ready(Self::READ_AREA)?;
        let size = self.buffer.buffer();
        if size.width == 0 || size.height == 0 { Err(AmmlStatus::NOT_READY) } else { Ok(size) }
    }

    pub fn cursor_pos(&self) -> Result<AmmlPoint, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        Self::cursor_in(hwnd).ok_or(AmmlStatus::FAILED)
    }

    /// `NOT_READY` while the back buffer size or the client size is not known.
    pub fn to_buffer(&self, client: AmmlPoint) -> Result<AmmlPoint, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        self.refresh_client(hwnd);
        self.buffer.to_buffer(client).ok_or(AmmlStatus::NOT_READY)
    }

    /// Whether the window (or a child of it) has the keyboard focus of the window's thread.
    pub fn has_focus(&self) -> Result<bool, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        let window = Self::handle(hwnd);
        // SAFETY: a handle that is not a window gives thread 0, which the next call refuses.
        let thread = unsafe { GetWindowThreadProcessId(window, None) };
        let mut info = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        // SAFETY: `info` is a writable local with `cbSize` set.
        if unsafe { GetGUIThreadInfo(thread, &mut info) }.is_err() {
            return Err(AmmlStatus::FAILED);
        }
        let focus = info.hwndFocus;
        // SAFETY: both handles are plain values; a handle that is not a window gives false.
        Ok(focus == window || unsafe { IsChild(window, focus) }.as_bool())
    }

    pub fn is_foreground(&self) -> Result<bool, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        // SAFETY: no input.
        Ok(unsafe { GetForegroundWindow() } == Self::handle(hwnd))
    }

    pub fn dpi(&self) -> Result<u32, AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        // SAFETY: a handle that is not a window gives 0.
        match unsafe { GetDpiForWindow(Self::handle(hwnd)) } {
            0 => Err(AmmlStatus::FAILED),
            dpi => Ok(dpi),
        }
    }

    /// Asks the game to render at `size` for the mod at `owner` by posting `WM_SIZE` / `SIZE_RESTORED`; `None`
    /// posts the client size, which gives the window its own size back. The mod that asked for a size last holds
    /// it, and its disable (`end_owner`) posts the client size again. A side of 0 or above 65535 is
    /// `INVALID_ARGUMENT`; a minimised window has no client size to give back (`FAILED`). While the loader shuts down
    /// it is `NOT_READY` and nothing is posted.
    pub fn request_size(&self, owner: usize, size: Option<AmmlSize>) -> Result<(), AmmlStatus> {
        let hwnd = self.ready(Self::REQUEST_SIZE_AREA)?;
        if self.shutdown.is_closing() {
            return Err(AmmlStatus::NOT_READY);
        }
        let (width, height, holds) = match size {
            Some(size) if Self::fits(size) => (size.width, size.height, true),
            Some(_) => return Err(AmmlStatus::INVALID_ARGUMENT),
            None => {
                let client = Self::client_rect(hwnd).filter(|client| Self::fits(*client)).ok_or(AmmlStatus::FAILED)?;
                (client.width, client.height, false)
            }
        };
        let reason = if holds { "requested" } else { "given back" };
        if self.post_size(owner, hwnd, width, height, holds, reason) { Ok(()) } else { Err(AmmlStatus::FAILED) }
    }

    /// Clips the cursor for the mod at `owner`, or drops its request with `None`. The clip is set only while the
    /// window is in front and not minimised; a loss of that keeps the request and gives the clip back. The last
    /// request wins and the others come back when it drops. A request while the loader shuts down is `NOT_READY`.
    ///
    /// `live` tells whether the mod may still hold a clip; it runs under the clip lock for a `Some` request, so no
    /// request lands after `end_owner` of that mod (`REFUSED`).
    pub fn clip_cursor(
        &self,
        owner: usize,
        clip: Option<ClipKind>,
        live: impl FnOnce() -> bool,
    ) -> Result<(), AmmlStatus> {
        let hwnd = self.ready(Self::CURSOR_CLIP_AREA)?;
        if clip.is_some() && self.shutdown.is_closing() {
            return Err(AmmlStatus::NOT_READY);
        }
        let shutdown = &self.shutdown;
        if self.clip.request(hwnd, owner, clip, || live() && !shutdown.is_closing()) {
            Ok(())
        } else {
            Err(AmmlStatus::REFUSED)
        }
    }

    /// Ends what the mod at `owner` holds in the window: its cursor clip request is dropped, and when it holds the
    /// render size, the client size is posted.
    /// While the loader shuts down nothing is posted: the message would reach a game that tears its renderer down.
    pub fn end_owner(&self, owner: usize) {
        self.clip.end_owner(self.hwnd(), owner);
        if !self.sizes.end_owner(owner) {
            return;
        }
        if self.shutdown.is_closing() {
            curia::info!("render size not restored: the loader shuts down", { owner: owner });
            return;
        }
        let Some(hwnd) = self.hwnd() else {
            return;
        };
        match Self::client_rect(hwnd).filter(|client| Self::fits(*client)) {
            Some(client) => {
                self.post_size(owner, hwnd, client.width, client.height, false, "restore");
            }
            None => curia::warn!("render size not restored: the window has no client size", { owner: owner }),
        }
    }

    /// Posts the focus losses listeners held back to the window again. They pass the loader's procedure once,
    /// raised with `replayed = 1`, and reach the game whatever listeners say.
    pub fn replay_focus_loss(&self) -> Result<(), AmmlStatus> {
        let hwnd = self.ready(Self::READ_AREA)?;
        for message in self.focus.take_replay() {
            // SAFETY: a handle that is not a window makes the call fail, and the message carries no pointer.
            let posted = unsafe {
                PostMessageW(Some(Self::handle(hwnd)), message.msg, WPARAM(message.wparam), LPARAM(message.lparam))
            };
            if let Err(error) = posted {
                curia::warn!("a held focus loss was not replayed", { message: message.msg, error: error.to_string() });
            }
        }
        Ok(())
    }

    // What the procedure kept between messages of the old window; the render size a mod holds stays.
    fn forget_window_state(&self) {
        self.pairs.reset();
        self.focus.gain();
        self.sizes.forget_pending();
        self.router.surrogates().reset();
    }

    // Records the post for the window procedure, then posts; a failed post is forgotten again.
    fn post_size(&self, owner: usize, hwnd: usize, width: u32, height: u32, holds: bool, reason: &'static str) -> bool {
        let lparam = SizeMessage::size_param(width, height);
        self.sizes.posting(owner, lparam, holds);
        let posted = SizeMessage::post(hwnd, width, height, reason);
        if !posted {
            self.sizes.post_failed(owner, lparam);
        }
        posted
    }

    // The attached window, when `area` is offered.
    fn ready(&self, area: &str) -> Result<usize, AmmlStatus> {
        if !self.areas.is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.hwnd().ok_or(AmmlStatus::NOT_READY)
    }

    fn fits(size: AmmlSize) -> bool {
        (1..=Self::MAX_SIDE).contains(&size.width) && (1..=Self::MAX_SIDE).contains(&size.height)
    }

    /// The handle of the window `hwnd`.
    pub fn handle(hwnd: usize) -> HWND {
        HWND(std::ptr::with_exposed_provenance_mut::<c_void>(hwnd))
    }

    fn has_mark(window: HWND) -> bool {
        // SAFETY: a handle that is not a window gives a null value; the property name is a static string.
        !unsafe { GetPropW(window, Self::MARK) }.is_invalid()
    }

    fn is_window(hwnd: usize) -> bool {
        // SAFETY: a handle that is not a window gives false.
        unsafe { IsWindow(Some(Self::handle(hwnd))) }.as_bool()
    }

    fn client_rect(hwnd: usize) -> Option<AmmlSize> {
        let mut rect = RECT::default();
        // SAFETY: `rect` is a writable local; a handle that is not a window makes the call fail.
        unsafe { GetClientRect(Self::handle(hwnd), &mut rect) }.ok()?;
        Some(AmmlSize { width: rect.right.max(0) as u32, height: rect.bottom.max(0) as u32 })
    }

    /// The cursor position in the client coordinates of `hwnd`.
    pub fn cursor_in(hwnd: usize) -> Option<AmmlPoint> {
        let mut point = POINT::default();
        // SAFETY: `point` is a writable local.
        unsafe { GetCursorPos(&mut point) }.ok()?;
        Self::screen_to_client(hwnd, point)
    }

    /// The screen point `point` in the client coordinates of `hwnd`.
    pub fn screen_to_client(hwnd: usize, mut point: POINT) -> Option<AmmlPoint> {
        // SAFETY: `point` is a writable local; a handle that is not a window makes the call fail.
        if !unsafe { ScreenToClient(Self::handle(hwnd), &mut point) }.as_bool() {
            return None;
        }
        Some(AmmlPoint { x: point.x as f32, y: point.y as f32 })
    }
}

use amml_abi::{AmmlPoint, AmmlSize, AmmlStatus, AmmlStr, AmmlWindowTable};

use crate::{ApiError, Point, Size};

/// The game window, through the loader's window table. Usable from any thread.
///
/// The reads give `None` while the loader cannot answer: no window table (a loader before v3), the
/// area `window.read` not offered (no overlay hooks, the LeviLamina host), or no window attached yet
/// (before the first present of the game's swap chain).
#[derive(Clone, Copy)]
pub struct Window {
    table: Option<AmmlWindowTable>,
    module: &'static str,
}

impl Window {
    pub(crate) fn new(table: Option<AmmlWindowTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// The client area size; `None` also while the window is minimised.
    pub fn client_size(&self) -> Option<Size> {
        self.read(AmmlSize { width: 0, height: 0 }, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.client_size)(out)
            }
        })
        .map(Size::from)
    }

    /// The back buffer size of the game's swap chain; `None` until the loader bound it.
    pub fn buffer_size(&self) -> Option<Size> {
        self.read(AmmlSize { width: 0, height: 0 }, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.buffer_size)(out)
            }
        })
        .map(Size::from)
    }

    /// The cursor position in client coordinates.
    pub fn cursor_pos(&self) -> Option<Point> {
        self.read(AmmlPoint { x: 0.0, y: 0.0 }, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.cursor_pos)(out)
            }
        })
        .map(Point::from)
    }

    /// `client` in back buffer coordinates, scaled by buffer size over client size; `None` until both
    /// sizes are known and while either has a side of 0.
    pub fn to_buffer(&self, client: Point) -> Option<Point> {
        self.read(AmmlPoint { x: 0.0, y: 0.0 }, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.to_buffer)(AmmlPoint::from(client), out)
            }
        })
        .map(Point::from)
    }

    /// Whether the game window (or a child of it) has the keyboard focus.
    pub fn has_focus(&self) -> Option<bool> {
        self.read(0u8, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.has_focus)(out)
            }
        })
        .map(|focus| focus != 0)
    }

    /// Whether the game window is the foreground window.
    pub fn is_foreground(&self) -> Option<bool> {
        self.read(0u8, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.is_foreground)(out)
            }
        })
        .map(|foreground| foreground != 0)
    }

    /// The DPI of the game window (96 at 100 %).
    pub fn dpi(&self) -> Option<u32> {
        self.read(0u32, |table, out| {
            // SAFETY: `out` is a live local for the call.
            #[allow(unsafe_code)]
            unsafe {
                (table.dpi)(out)
            }
        })
    }

    /// Asks the game for a client size: `Some` sets it and makes this mod the requester; `None` gives
    /// the window's own client size back. The loader gives the size back when the requester is disabled
    /// (not during shutdown). The resulting `WindowSize` has `is_injected() == true`. A side of 0 or above
    /// 65535 gives `Loader(INVALID_ARGUMENT)`; without the area `window.request_size`, `NotAvailable`;
    /// before the window is attached, `NotReady`.
    pub fn request_size(&self, size: Option<Size>) -> Result<(), ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let size = size.map(AmmlSize::from);
        let pointer = size.as_ref().map_or(std::ptr::null(), std::ptr::from_ref);
        // SAFETY: the module id lives for the call, and a non-null `pointer` points to the local `size`.
        #[allow(unsafe_code)]
        let status = unsafe { (table.request_size)(AmmlStr::from_str(self.module), pointer) };
        ApiError::check(status)
    }

    /// Passes the focus losses a `WindowFocus` listener held back on to the game, each raised again with
    /// `replayed() == true`; a cancel of a replay is ignored. Nothing is held after the window gained the
    /// focus again. `NotAvailable` without the area `window.read`; `NotReady` before the window is
    /// attached.
    pub fn replay_focus_loss(&self) -> Result<(), ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader's function takes no arguments.
        #[allow(unsafe_code)]
        let status = unsafe { (table.replay_focus_loss)() };
        ApiError::check(status)
    }

    // Runs one read into `out`; `None` without a table or for any status but `OK`.
    fn read<T>(&self, mut out: T, call: impl FnOnce(&AmmlWindowTable, *mut T) -> AmmlStatus) -> Option<T> {
        let table = self.table.as_ref()?;
        call(table, &mut out).is_ok().then_some(out)
    }
}

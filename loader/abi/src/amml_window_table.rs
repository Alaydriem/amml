use crate::{AmmlCursorClip, AmmlPoint, AmmlSize, AmmlStatus, AmmlStr};

/// Gives the client area size of the game window in `out`.
pub type WindowClientSizeFn = unsafe extern "C" fn(out: *mut AmmlSize) -> AmmlStatus;
/// Gives the back buffer size of the bound swap chain in `out`.
pub type WindowBufferSizeFn = unsafe extern "C" fn(out: *mut AmmlSize) -> AmmlStatus;
/// Gives the cursor position in client coordinates in `out`.
pub type WindowCursorPosFn = unsafe extern "C" fn(out: *mut AmmlPoint) -> AmmlStatus;
/// Maps a point in client coordinates to back buffer coordinates in `out`.
pub type WindowToBufferFn = unsafe extern "C" fn(client: AmmlPoint, out: *mut AmmlPoint) -> AmmlStatus;
/// Gives 1 in `out` while the game window has the keyboard focus.
pub type WindowHasFocusFn = unsafe extern "C" fn(out: *mut u8) -> AmmlStatus;
/// Gives 1 in `out` while the game window is the foreground window.
pub type WindowIsForegroundFn = unsafe extern "C" fn(out: *mut u8) -> AmmlStatus;
/// Gives the DPI of the game window in `out`.
pub type WindowDpiFn = unsafe extern "C" fn(out: *mut u32) -> AmmlStatus;
/// Asks for the client size `size` for `module`; a null `size` gives the window its own size back.
pub type WindowRequestSizeFn = unsafe extern "C" fn(module: AmmlStr, size: *const AmmlSize) -> AmmlStatus;
/// Passes the focus loss a listener held back on to the game, marked replayed.
pub type WindowReplayFocusLossFn = unsafe extern "C" fn() -> AmmlStatus;
/// Asks the loader to clip the cursor for `module`, or gives the clip back (`AmmlCursorClip::NONE`).
pub type WindowClipCursorFn = unsafe extern "C" fn(module: AmmlStr, clip: AmmlCursorClip) -> AmmlStatus;

/// The window functions of `AmmlServices`. Each gives `NOT_READY` before the loader attached the game window. The
/// table only grows: a function added later is appended, `size` and `version` grow, and a reader calls a function
/// only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlWindowTable {
    pub size: u32,
    pub version: u32,
    pub client_size: WindowClientSizeFn,
    pub buffer_size: WindowBufferSizeFn,
    pub cursor_pos: WindowCursorPosFn,
    pub to_buffer: WindowToBufferFn,
    pub has_focus: WindowHasFocusFn,
    pub is_foreground: WindowIsForegroundFn,
    pub dpi: WindowDpiFn,
    pub request_size: WindowRequestSizeFn,
    pub replay_focus_loss: WindowReplayFocusLossFn,
    pub clip_cursor: Option<WindowClipCursorFn>,
}

impl AmmlWindowTable {
    /// `size` of a v1 window table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 80;
    /// `size` of a v2 window table: v1 plus `clip_cursor`.
    pub const SIZE_V2: u32 = 88;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null
    /// table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlWindowTable) -> Option<AmmlWindowTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlWindowTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlWindowTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlWindowTable>() == AmmlWindowTable::SIZE_V2 as usize);

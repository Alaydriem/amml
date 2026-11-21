use crate::{AmmlPoint, AmmlSize, AmmlStatus, AmmlStr};

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

/// The window functions of `AmmlServices`. Each gives `NOT_READY` before the loader attached the game window.
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
}

impl AmmlWindowTable {
    /// `size` of a v1 window table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 80;
}

const _: () = assert!(core::mem::size_of::<AmmlWindowTable>() == AmmlWindowTable::SIZE_V1 as usize);

use amml_abi::{AmmlPoint, AmmlSize, AmmlStatus, AmmlStr, AmmlWindowTable};

use crate::{Boundary, LoaderState, WindowService};

const TABLE_VERSION: u32 = 1;

pub(crate) static WINDOW: AmmlWindowTable = AmmlWindowTable {
    size: AmmlWindowTable::SIZE_V1,
    version: TABLE_VERSION,
    client_size: WindowTable::client_size,
    buffer_size: WindowTable::buffer_size,
    cursor_pos: WindowTable::cursor_pos,
    to_buffer: WindowTable::to_buffer,
    has_focus: WindowTable::has_focus,
    is_foreground: WindowTable::is_foreground,
    dpi: WindowTable::dpi,
    request_size: WindowTable::request_size,
    replay_focus_loss: WindowTable::replay_focus_loss,
};

/// The window table behind `AmmlServices::window`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `WindowService` inside the boundary, so a panic never crosses it.
pub struct WindowTable;

impl WindowTable {
    unsafe extern "C" fn client_size(out: *mut AmmlSize) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, WindowService::client_size) }
    }

    unsafe extern "C" fn buffer_size(out: *mut AmmlSize) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, WindowService::buffer_size) }
    }

    unsafe extern "C" fn cursor_pos(out: *mut AmmlPoint) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, WindowService::cursor_pos) }
    }

    unsafe extern "C" fn to_buffer(client: AmmlPoint, out: *mut AmmlPoint) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, |window| window.to_buffer(client)) }
    }

    unsafe extern "C" fn has_focus(out: *mut u8) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, |window| window.has_focus().map(u8::from)) }
    }

    unsafe extern "C" fn is_foreground(out: *mut u8) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, |window| window.is_foreground().map(u8::from)) }
    }

    unsafe extern "C" fn dpi(out: *mut u32) -> AmmlStatus {
        // SAFETY: the caller promises a non-null `out` is writable.
        unsafe { Self::read(out, WindowService::dpi) }
    }

    unsafe extern "C" fn request_size(module: AmmlStr, size: *const AmmlSize) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: the caller promises a non-null `size` points to a readable `AmmlSize`.
            let size = (!size.is_null()).then(|| unsafe { size.read() });
            Self::status(state.window().request_size(owner, size))
        })
    }

    unsafe extern "C" fn replay_focus_loss() -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => Self::status(state.window().replay_focus_loss()),
            None => AmmlStatus::FAILED,
        })
    }

    fn status(result: Result<(), AmmlStatus>) -> AmmlStatus {
        match result {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // Runs one read and writes its value to `out`.
    //
    // Safety: a non-null `out` is writable for one `T`.
    unsafe fn read<T>(out: *mut T, read: impl FnOnce(&WindowService) -> Result<T, AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match read(state.window()) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }
}

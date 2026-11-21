use amml_abi::AmmlCursorShape;
use windows::Win32::UI::WindowsAndMessaging::{IDC_ARROW, IDC_HAND, IDC_IBEAM, LoadCursorW, SetCursor};
use windows::core::PCWSTR;

/// The system cursors behind `AmmlCursorShape`.
pub struct CursorShapes;

impl CursorShapes {
    /// The system cursor of `shape`; `None` for `NONE` and for a value the loader does not know.
    pub fn system_id(shape: AmmlCursorShape) -> Option<PCWSTR> {
        match shape {
            AmmlCursorShape::ARROW => Some(IDC_ARROW),
            AmmlCursorShape::HAND => Some(IDC_HAND),
            AmmlCursorShape::TEXT => Some(IDC_IBEAM),
            _ => None,
        }
    }

    /// Sets the cursor to `shape`. False when the shape has no system cursor or Windows refused it.
    pub fn apply(shape: AmmlCursorShape) -> bool {
        let Some(id) = Self::system_id(shape) else {
            return false;
        };
        // SAFETY: a system cursor is shared by the system and never freed.
        let Ok(cursor) = (unsafe { LoadCursorW(None, id) }) else {
            return false;
        };
        // SAFETY: the cursor is a live system cursor.
        unsafe { SetCursor(Some(cursor)) };
        true
    }
}

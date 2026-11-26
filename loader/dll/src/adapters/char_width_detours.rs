use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, InlineIconAdapter, ScreenDetours, SessionHook, UiFlags};

/// `float BitmapFont::_getCharWidth(int character, bool forceUnicode)`: rcx the font, edx the character, r8b the flag;
/// the width in xmm0. Nothing calls it directly: the game calls it through a vtable slot, and the hook sits on the
/// function itself.
type CharWidthFn = unsafe extern "C" fn(usize, i32, u8) -> f32;

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour on the width of a character of a bitmap font.
///
/// With `UiFlags::ICONS` clear (no inline icon is registered) the detour does one relaxed load of `UiFlags` and calls the
/// original with the registers it received. With it set, a character that has an icon gets the advance of the icon and
/// every other character goes to the original. No call leaves the loader.
pub struct CharWidthDetours;

impl CharWidthDetours {
    pub const ENTRY: &'static str = "BitmapFont._getCharWidth";

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: None,
            detour: Self::char_width as CharWidthFn as *const c_void,
            original: &ORIGINAL,
        }
    }

    unsafe extern "C" fn char_width(font: usize, character: i32, force_unicode: u8) -> f32 {
        if UiFlags::load() & UiFlags::ICONS != 0
            && let Some(width) =
                Boundary::run(None, || InlineIconAdapter::with(None, |adapter| adapter.width(character)))
        {
            return width;
        }
        // SAFETY: the hook on `BitmapFont._getCharWidth` is installed, so its slot holds a `CharWidthFn`.
        let original = unsafe { ScreenDetours::original::<CharWidthFn>(&ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(font, character, force_unicode) }
    }
}

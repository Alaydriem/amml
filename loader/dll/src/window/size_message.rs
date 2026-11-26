use std::ffi::c_void;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, SIZE_RESTORED, WM_SIZE};

/// A `WM_SIZE` posted to the game window, so the game takes a new render size the way it takes a window resize.
pub struct SizeMessage;

impl SizeMessage {
    /// Posts `WM_SIZE` / `SIZE_RESTORED` with `width` and `height` to the window `hwnd`, and logs it with
    /// `reason`. False for a null window, a side of 0 or a post Windows refused (logged).
    pub fn post(hwnd: usize, width: u32, height: u32, reason: &'static str) -> bool {
        if hwnd == 0 || width == 0 || height == 0 {
            return false;
        }
        let window = HWND(std::ptr::with_exposed_provenance_mut::<c_void>(hwnd));
        let lparam = LPARAM(Self::size_param(width, height));
        // SAFETY: a handle that is not a window makes the call fail, and the message carries no pointer.
        match unsafe { PostMessageW(Some(window), WM_SIZE, WPARAM(SIZE_RESTORED as usize), lparam) } {
            Ok(()) => {
                curia::info!("render size message posted", { width: width, height: height, reason: reason });
                true
            }
            Err(error) => {
                curia::warn!("render size message not posted", { error: error.to_string(), reason: reason });
                false
            }
        }
    }

    /// The `lParam` of a `WM_SIZE` of `width` x `height`: the width in the low word, the height in the high word.
    pub fn size_param(width: u32, height: u32) -> isize {
        (((height & 0xFFFF) << 16) | (width & 0xFFFF)) as isize
    }

    /// The width and height a `WM_SIZE` `lParam` carries.
    pub fn size_of(lparam: isize) -> (u32, u32) {
        ((lparam & 0xFFFF) as u32, ((lparam >> 16) & 0xFFFF) as u32)
    }
}

use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, ScreenDetours, SessionHook, UiFlags, UiInputAdapter};

/// `void ScreenView::handleButtonEvent(uint buttonId, ButtonState state, FocusImpact impact)`: rcx the view, edx the
/// button id, r8b the state (1 pressed, 0 released), r9b the focus impact; no result.
type ButtonFn = unsafe extern "C" fn(*mut c_void, u32, u8, u8);

/// `void UIScene::handleDirection(DirectionId, float x, float y, FocusImpact)`: rcx the scene, dl the direction
/// (Scroll is 2), xmm2 x, xmm3 y, the focus impact on the stack; no result.
type DirectionFn = unsafe extern "C" fn(*mut c_void, u8, f32, f32, u8);

static BUTTON_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static DIRECTION_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the button handler of a screen view and the direction handler of a scene.
///
/// Each detour loads `UiFlags` once. With its bit clear (no listener of `UiPointer`, or of `UiScroll`) it calls the
/// original with the registers it received and returns. With the bit set it asks the adapter first and returns
/// without the original when the adapter says a listener cancelled the input: the game then does not see that one
/// button event or wheel step and nothing else.
pub struct UiInputDetours;

impl UiInputDetours {
    pub const BUTTON: &'static str = "ScreenView.handleButtonEvent";
    pub const DIRECTION: &'static str = "UIScene.handleDirection";
    /// The `DirectionId` of a wheel step.
    pub const SCROLL: u8 = 2;

    pub fn button_hook() -> SessionHook {
        SessionHook {
            entry: Self::BUTTON,
            vtable: None,
            detour: Self::button as ButtonFn as *const c_void,
            original: &BUTTON_ORIGINAL,
        }
    }

    pub fn direction_hook() -> SessionHook {
        SessionHook {
            entry: Self::DIRECTION,
            vtable: None,
            detour: Self::direction as DirectionFn as *const c_void,
            original: &DIRECTION_ORIGINAL,
        }
    }

    /// Sets whether a listener of `UiPointer` exists.
    pub(crate) fn set_pointer_wanted(wanted: bool) {
        if !wanted {
            UiInputAdapter::with((), UiInputAdapter::forget_pointers);
        }
        UiFlags::put(UiFlags::POINTER, wanted);
    }

    /// Sets whether a listener of `UiScroll` exists.
    pub(crate) fn set_scroll_wanted(wanted: bool) {
        UiFlags::put(UiFlags::SCROLL, wanted);
    }

    unsafe extern "C" fn button(this: *mut c_void, id: u32, state: u8, impact: u8) {
        if UiFlags::load() & UiFlags::POINTER != 0 {
            let skip = Boundary::run(false, || {
                UiInputAdapter::with(false, |adapter| adapter.pointer(this.expose_provenance(), id, state != 0))
            });
            if skip {
                return;
            }
        }
        // SAFETY: the hook on `ScreenView.handleButtonEvent` is installed, so its slot holds a `ButtonFn`.
        let original = unsafe { ScreenDetours::original::<ButtonFn>(&BUTTON_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, id, state, impact) };
    }

    unsafe extern "C" fn direction(this: *mut c_void, direction: u8, x: f32, y: f32, impact: u8) {
        if direction == Self::SCROLL && UiFlags::load() & UiFlags::SCROLL != 0 {
            let skip = Boundary::run(false, || {
                UiInputAdapter::with(false, |adapter| adapter.scroll(this.expose_provenance(), y))
            });
            if skip {
                return;
            }
        }
        // SAFETY: the hook on `UIScene.handleDirection` is installed, so its slot holds a `DirectionFn`.
        let original = unsafe { ScreenDetours::original::<DirectionFn>(&DIRECTION_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, direction, x, y, impact) };
    }
}

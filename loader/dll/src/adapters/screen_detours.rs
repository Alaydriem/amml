use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, ScreenAdapter, SessionHook, TexturePump, UiFlags};

/// `void ScreenView::render(UIRenderContext&)`: rcx the view, rdx the render context; no result.
type RenderFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// `ViewRequest ContainerScreenController::_onContainerSlotHovered(std::string const&, int)` and its unhover twin: rcx
/// the controller, rdx the collection name, r8d the slot; the enum result in eax.
type HoverFn = unsafe extern "C" fn(*mut c_void, *const c_void, i32) -> u32;

static RENDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static HOVER_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static UNHOVER_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the draw of a screen view and on the hover and unhover handlers of a container screen.
/// The draw detour loads `UiFlags` once. With the word 0 (no listener of `ScreenRender` or `ScreenChanged`, no
/// texture work) it calls the original and returns. Otherwise it runs the texture pump when the texture bits say so,
/// calls the original with the registers it received, and tells the screen adapter, with the render context, while a
/// listener exists. A hover detour calls the original, then does one relaxed load of `UiFlags` and returns unless
/// the hover bit is set.
pub struct ScreenDetours;

impl ScreenDetours {
    pub const RENDER: &'static str = "ScreenView.render";
    pub const HOVER: &'static str = "ContainerScreenController.onContainerSlotHovered";
    pub const UNHOVER: &'static str = "ContainerScreenController.onContainerSlotUnhovered";

    pub fn render_hook() -> SessionHook {
        SessionHook {
            entry: Self::RENDER,
            vtable: None,
            detour: Self::render as RenderFn as *const c_void,
            original: &RENDER,
        }
    }

    /// Sets whether a listener of `ScreenRender` or `ScreenChanged` exists.
    pub(crate) fn set_views_wanted(wanted: bool) {
        UiFlags::put(UiFlags::VIEWS, wanted);
    }

    /// Whether a listener of `ScreenRender` or `ScreenChanged` exists.
    pub fn views_wanted() -> bool {
        UiFlags::load() & UiFlags::VIEWS != 0
    }

    /// Sets whether a listener of `ContainerSlotHover` or `ContainerSlotUnhover` exists.
    pub(crate) fn set_hover_wanted(wanted: bool) {
        UiFlags::put(UiFlags::HOVER, wanted);
    }

    /// Whether a listener of `ContainerSlotHover` or `ContainerSlotUnhover` exists.
    pub fn hover_wanted() -> bool {
        UiFlags::load() & UiFlags::HOVER != 0
    }

    pub fn hover_hook() -> SessionHook {
        SessionHook {
            entry: Self::HOVER,
            vtable: None,
            detour: Self::hover as HoverFn as *const c_void,
            original: &HOVER_ORIGINAL,
        }
    }

    pub fn unhover_hook() -> SessionHook {
        SessionHook {
            entry: Self::UNHOVER,
            vtable: None,
            detour: Self::unhover as HoverFn as *const c_void,
            original: &UNHOVER_ORIGINAL,
        }
    }

    // The container hover detours: with the hover bit clear they call the original and return after one relaxed load.
    // Otherwise they call the original with the registers they received, then tell the screen adapter, which raises
    // the event after the game handled the hover. The result is the game's own.
    unsafe extern "C" fn hover(this: *mut c_void, name: *const c_void, slot: i32) -> u32 {
        // SAFETY: the hook on the hover handler is installed, so its slot holds a `HoverFn`.
        let original = unsafe { Self::original::<HoverFn>(&HOVER_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, name, slot) };
        if UiFlags::load() & UiFlags::HOVER != 0 {
            Boundary::run((), || ScreenAdapter::with((), |adapter| adapter.hovered(Self::HOVER, this, name, slot)));
        }
        result
    }

    unsafe extern "C" fn unhover(this: *mut c_void, name: *const c_void, slot: i32) -> u32 {
        // SAFETY: the hook on the unhover handler is installed, so its slot holds a `HoverFn`.
        let original = unsafe { Self::original::<HoverFn>(&UNHOVER_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, name, slot) };
        if UiFlags::load() & UiFlags::HOVER != 0 {
            Boundary::run((), || ScreenAdapter::with((), |adapter| adapter.hovered(Self::UNHOVER, this, name, slot)));
        }
        result
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    pub(crate) unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the caller
        // guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    unsafe extern "C" fn render(this: *mut c_void, context: *mut c_void) {
        let flags = UiFlags::load();
        if flags & (UiFlags::DUE | UiFlags::WAITING) != 0 {
            Boundary::run((), || TexturePump::run(context.expose_provenance()));
        }
        // SAFETY: the hook on `ScreenView.render` is installed, so its slot holds a `RenderFn`.
        let original = unsafe { Self::original::<RenderFn>(&RENDER) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, context) };
        if flags & (UiFlags::VIEWS | UiFlags::HOVER) == 0 {
            return;
        }
        Boundary::run((), || ScreenAdapter::with((), |adapter| adapter.rendered(this, context)));
    }
}

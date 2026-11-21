use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, ScreenDetours, SessionHook, UiFlags, UiInputAdapter};

/// `void ShulkerBoxBlockItem::appendFormattedHovertext(ItemStackBase const&, Level&, RedactableString&, bool) const`
/// and the base `Item` function of the same shape: rcx the item, rdx the stack, r8 the level, r9 the text, the bool
/// on the stack; no result.
type HovertextFn = unsafe extern "C" fn(*mut c_void, *const c_void, *mut c_void, *mut c_void, u8);

static HOVERTEXT_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour on the hover text of a shulker box item.
///
/// With the `UiFlags` bit clear (no listener of `ItemHovertext`) the detour calls the original and returns after one
/// relaxed load. With it set the adapter raises the event first. When a listener asked to hide the contents the
/// detour calls the base `Item` function, with the registers it received, in place of the original: the name and
/// enchantment lines stay and the contents lines are never built.
pub struct HovertextDetours;

impl HovertextDetours {
    pub const HOVERTEXT: &'static str = "ShulkerBoxBlockItem.appendFormattedHovertext";
    /// The base function the shulker box override calls first.
    pub const BASE: &'static str = "Item.appendFormattedHovertext";

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::HOVERTEXT,
            vtable: None,
            detour: Self::hovertext as HovertextFn as *const c_void,
            original: &HOVERTEXT_ORIGINAL,
        }
    }

    /// Sets whether a listener of `ItemHovertext` exists.
    pub(crate) fn set_wanted(wanted: bool) {
        UiFlags::put(UiFlags::HOVERTEXT, wanted);
    }

    unsafe extern "C" fn hovertext(
        this: *mut c_void,
        stack: *const c_void,
        level: *mut c_void,
        text: *mut c_void,
        advanced: u8,
    ) {
        if UiFlags::load() & UiFlags::HOVERTEXT != 0
            && let Some(base) = Boundary::run(None, || UiInputAdapter::with(None, |adapter| adapter.hidden_base()))
        {
            // SAFETY: `base` is the address of `Item::appendFormattedHovertext`, which has the shape of the
            // override the game called, and the registers are the game's own.
            let base = unsafe { std::mem::transmute::<usize, HovertextFn>(base) };
            // SAFETY: the game's own arguments, passed on unchanged.
            unsafe { base(this, stack, level, text, advanced) };
            return;
        }
        // SAFETY: the hook on the override is installed, so its slot holds a `HovertextFn`.
        let original = unsafe { ScreenDetours::original::<HovertextFn>(&HOVERTEXT_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, stack, level, text, advanced) };
    }
}

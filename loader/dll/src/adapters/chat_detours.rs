use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use amml_data::AdapterCatalog;

use crate::{Boundary, ChatAdapter, ScreenDetours, SessionHook, UiFlags};

/// `GuiMessage::GuiMessage` on the 26.6x argument list: rcx this, edx the message type, r8 the username, r9 the message,
/// then on the stack a pointer to the filtered message, the TTS message, the author id, the lifetime, and two flags. It
/// returns `this`.
///
/// Every stack argument is declared as a whole 8-byte slot (`u64`), whatever its C++ type is: a narrower type would not
/// write the upper bytes of its slot when the detour calls the original, and the original may read them. A slot is
/// forwarded bit for bit.
type CtorFn = unsafe extern "C" fn(usize, i32, usize, usize, usize, usize, usize, u64, u64, u64) -> usize;
/// `GuiMessage::GuiMessage` on the 70.x argument list: the 26.6x list and one more stack slot, at `[rsp+58h]` on entry, that
/// the constructor stores as a word (the slots of the other arguments are the same as on 26.6x).
type FlagWordCtorFn = unsafe extern "C" fn(usize, i32, usize, usize, usize, usize, usize, u64, u64, u64, u64) -> usize;

static CTOR_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static FLAG_WORD_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the constructor of a chat line.
///
/// One constructor has two argument lists, and `GuiMessage.ctorShape` says which one a build has: 0 the 26.6x list
/// (entry `GuiMessage.ctor`), 1 the 70.x list (entry `GuiMessage.ctorWithFlagWord`). `GuiMessage.ctor` is refused as not for
/// this build on shape 1 (entry_is), and the loader still hooks only the entry of the shape.
///
/// Both detours call the original first. With `UiFlags::CHAT` set (a listener of `ChatMessageAdded` exists) they then
/// hand the new line to `ChatAdapter`, which raises the event while the constructor has just returned and the game has
/// not added the line to its list. With the bit clear a detour is the original call and one relaxed load.
pub struct ChatDetours;

impl ChatDetours {
    pub const CTOR: &'static str = "GuiMessage.ctor";
    pub const FLAG_WORD_CTOR: &'static str = "GuiMessage.ctorWithFlagWord";
    /// The value of `GuiMessage.ctorShape` of the 26.6x argument list.
    pub const SHAPE_26_6X: u64 = 0;
    /// The value of `GuiMessage.ctorShape` of the 70.x argument list.
    pub const SHAPE_70_X: u64 = 1;

    /// The entry to hook for the value of `GuiMessage.ctorShape`; `None` for a value no detour has.
    pub fn target_for(shape: u64) -> Option<&'static str> {
        match shape {
            Self::SHAPE_26_6X => Some(Self::CTOR),
            Self::SHAPE_70_X => Some(Self::FLAG_WORD_CTOR),
            _ => None,
        }
    }

    /// The name of the catalog shape of `ui.chat_message` that goes with the value of `GuiMessage.ctorShape`.
    pub fn catalog_shape_for(shape: u64) -> Option<&'static str> {
        match shape {
            Self::SHAPE_26_6X => Some(AdapterCatalog::CHAT_CTOR_SHAPE),
            Self::SHAPE_70_X => Some(AdapterCatalog::CHAT_FLAG_WORD_SHAPE),
            _ => None,
        }
    }

    /// The hook for the value of `GuiMessage.ctorShape`; `None` for a value no detour has.
    pub fn hook_for(shape: u64) -> Option<SessionHook> {
        let (detour, original): (*const c_void, &'static AtomicPtr<c_void>) = match shape {
            Self::SHAPE_26_6X => (Self::ctor as CtorFn as *const c_void, &CTOR_ORIGINAL),
            Self::SHAPE_70_X => (Self::flag_word_ctor as FlagWordCtorFn as *const c_void, &FLAG_WORD_ORIGINAL),
            _ => return None,
        };
        Some(SessionHook { entry: Self::target_for(shape)?, vtable: None, detour, original })
    }

    /// Sets whether a listener of `ChatMessageAdded` exists.
    pub(crate) fn set_wanted(wanted: bool) {
        UiFlags::put(UiFlags::CHAT, wanted);
    }

    // Raises the event for the line the constructor just built at `message`.
    fn added(message: usize) {
        if UiFlags::load() & UiFlags::CHAT != 0 {
            Boundary::run((), || ChatAdapter::with((), |adapter| adapter.added(message)));
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn ctor(
        this: usize,
        kind: i32,
        username: usize,
        message: usize,
        filtered: usize,
        tts: usize,
        author: usize,
        lifetime: u64,
        force_visible: u64,
        tts_required: u64,
    ) -> usize {
        // SAFETY: the hook on `GuiMessage.ctor` is installed, so its slot holds a `CtorFn`.
        let original = unsafe { ScreenDetours::original::<CtorFn>(&CTOR_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe {
            original(this, kind, username, message, filtered, tts, author, lifetime, force_visible, tts_required)
        };
        Self::added(this);
        result
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn flag_word_ctor(
        this: usize,
        kind: i32,
        username: usize,
        message: usize,
        filtered: usize,
        tts: usize,
        author: usize,
        lifetime: u64,
        force_visible: u64,
        tts_required: u64,
        flag_word: u64,
    ) -> usize {
        // SAFETY: the hook on `GuiMessage.ctorWithFlagWord` is installed, so its slot holds a `FlagWordCtorFn`.
        let original = unsafe { ScreenDetours::original::<FlagWordCtorFn>(&FLAG_WORD_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe {
            original(
                this,
                kind,
                username,
                message,
                filtered,
                tts,
                author,
                lifetime,
                force_visible,
                tts_required,
                flag_word,
            )
        };
        Self::added(this);
        result
    }
}

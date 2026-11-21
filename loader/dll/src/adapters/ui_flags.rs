use std::sync::atomic::{AtomicU32, Ordering};

// The bits of the word.
static WORD: AtomicU32 = AtomicU32::new(0);

/// One word that tells the `ScreenView.render` detour, the container hover detours, the UI input detours, the packet
/// detours, the locator bar detours, the chat constructor detours and the font width detour whether they have anything to do. With no listener and nothing pending the word is 0 and a detour does one relaxed load and returns. Each
/// writer sets and clears only its own bit, with `fetch_or` and `fetch_and`, so no writer undoes another.
pub struct UiFlags;

impl UiFlags {
    /// A listener of `ScreenRender` or `ScreenChanged` exists (written by `EventHub`).
    pub const VIEWS: u32 = 1;
    /// The texture book has work that is due now (a request, an unload).
    pub const DUE: u32 = 2;
    /// A texture waits for the game; the poll time is read only when this bit is set.
    pub const WAITING: u32 = 4;
    /// A listener of `ContainerSlotHover` or `ContainerSlotUnhover` exists (written by `EventHub`).
    pub const HOVER: u32 = 8;
    /// A listener of `UiPointer` exists (written by `EventHub`).
    pub const POINTER: u32 = 16;
    /// A listener of `UiScroll` exists (written by `EventHub`).
    pub const SCROLL: u32 = 32;
    /// A listener of `ItemHovertext` exists (written by `EventHub`).
    pub const HOVERTEXT: u32 = 64;
    /// An inventory request runs, or a window for a hidden server answer is open (written by `RequestAdapter`).
    pub const REQUEST: u32 = 128;
    /// A listener of `PlayerListUpdate` exists (written by `EventHub`).
    pub const PLAYER_LIST: u32 = 256;
    /// A listener of `PlayerSkinChanged` exists (written by `EventHub`).
    pub const PLAYER_SKIN: u32 = 512;
    /// A listener of `LocatorBarIconDraw` exists (written by `EventHub`).
    pub const LOCATOR: u32 = 1024;
    /// The waypoint colours of the locator bar are tagged now, between the start and the end of one
    /// `HudLocatorBarRenderer.render` call (written by `LocatorBarAdapter`).
    pub const LOCATOR_TAGGED: u32 = 2048;
    /// A listener of `ChatMessageAdded` exists (written by `EventHub`).
    pub const CHAT: u32 = 4096;
    /// An inline chat icon is registered, so the width hook of the font has something to answer (written by
    /// `InlineIconAdapter`).
    pub const ICONS: u32 = 8192;

    /// The word: one relaxed load.
    pub fn load() -> u32 {
        WORD.load(Ordering::Relaxed)
    }

    /// Sets `bits`.
    pub fn set(bits: u32) {
        WORD.fetch_or(bits, Ordering::SeqCst);
    }

    /// Clears `bits`.
    pub fn clear(bits: u32) {
        WORD.fetch_and(!bits, Ordering::SeqCst);
    }

    /// Sets or clears `bits` by `on`.
    pub fn put(bits: u32, on: bool) {
        if on {
            Self::set(bits);
        } else {
            Self::clear(bits);
        }
    }
}

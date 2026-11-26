use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use amml_runtime::TextureBook;

use crate::{InlineIconAdapter, ItemDrawAdapter, TextureAdapter, UiFlags};

// The time, in milliseconds of `now_ms`, when the textures that wait for the game are polled next. It is read only
// while `UiFlags::WAITING` is set.
static NEXT_POLL: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();

/// Runs the game texture calls of `TextureAdapter` at a UI draw point: the start of a `ScreenView.render` detour (`run`:
/// binds, uploads and frees) and the start of a `HudLocatorBarRenderer.render` detour (`run_uploads`: binds and uploads
/// only; frees wait for the next `ScreenView.render`). Both detours call the pump when `UiFlags` says it has work. A
/// draw point with no work costs the one relaxed load of
/// `UiFlags` that the detour does anyway; with textures waiting it adds one load and a clock read. It takes no lock
/// and allocates nothing.
///
/// The calls need the render context of the draw point, so they never run from a table call: those only queue and
/// call `request`. Work that was just requested runs at the next draw point. A texture that the game has not given
/// yet is polled at most once in `TextureBook::REASK_MS`, so a texture that never becomes ready costs one pump a
/// second, not one for every frame.
pub struct TexturePump;

impl TexturePump {
    /// Tells the pump that the book has work. Call it after the change of the book.
    pub fn request() {
        UiFlags::set(UiFlags::DUE);
    }

    /// Milliseconds since the first call.
    pub fn now_ms() -> u64 {
        u64::try_from(START.get_or_init(Instant::now).elapsed().as_millis()).unwrap_or(u64::MAX - 1)
    }

    /// Whether the next draw point has work: a request, or a poll that is due.
    pub fn pending() -> bool {
        let flags = UiFlags::load();
        if flags & UiFlags::DUE != 0 {
            return true;
        }
        flags & UiFlags::WAITING != 0 && Self::now_ms() >= NEXT_POLL.load(Ordering::Relaxed)
    }

    /// After a pump: clears the flags, then asks `check` for the state of the book as `(work is due, textures
    /// wait)`. The flags are cleared before the book is read, so a `request` that comes from another thread at any
    /// time keeps the pump pending.
    pub fn settle(check: impl FnOnce() -> (bool, bool)) {
        UiFlags::clear(UiFlags::DUE | UiFlags::WAITING);
        let (due, waiting) = check();
        if due {
            UiFlags::set(UiFlags::DUE);
        }
        if waiting {
            NEXT_POLL.store(Self::now_ms().saturating_add(TextureBook::REASK_MS), Ordering::SeqCst);
            UiFlags::set(UiFlags::WAITING);
        }
    }

    /// At the draw point of a bar the game draws inside a screen view (`HudLocatorBarRenderer.render`) with the render
    /// context `context`: runs the binds and the uploads of the book, if there are any, and gives nothing back. The
    /// game may still hold a texture it drew earlier in the same frame, so a free waits for the next `run`, which is at
    /// the start of a `ScreenView.render`. Item draw frees wait too.
    pub fn run_uploads(context: usize) {
        if !Self::pending() {
            return;
        }
        TextureAdapter::with((), |adapter| adapter.pump_with(context, false));
    }

    /// At a draw point with the render context `context`: runs the work of the book, if there is any.
    pub fn run(context: usize) {
        if !Self::pending() {
            return;
        }
        ItemDrawAdapter::with((), ItemDrawAdapter::pump);
        InlineIconAdapter::with((), InlineIconAdapter::sync_page);
        if TextureAdapter::get().is_none() {
            Self::settle(|| (ItemDrawAdapter::with(false, ItemDrawAdapter::has_frees), false));
            return;
        }
        TextureAdapter::with((), |adapter| adapter.pump(context));
    }
}

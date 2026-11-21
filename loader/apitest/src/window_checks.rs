use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use amml_api::amml_abi::{SwapChainResized, WindowChar, WindowFocus, WindowPointer, WindowSize};
use amml_api::{
    Cancel, Event, EventCtx, Logger, MouseButton, PointerKind, Size, WindowCharExt, WindowFocusExt, WindowPointerExt,
    WindowSizeExt,
};

use crate::{Progress, Step};

/// The window checks of the API test mod: the `WindowPointer`, `WindowChar`, `WindowSize` and
/// `WindowFocus` events and the `SwapChainResized` of the in-game task's resizes, minimise, restore,
/// click and character, and the `Shutdown` at the window close. The listeners only use atomics, so
/// they never wait on the game thread.
pub struct WindowChecks {
    progress: &'static Progress,
    // Set once the overlay checks are done: the window events after it come from the in-game task.
    armed: AtomicBool,
    // The first left button down after `armed` is cancelled, and the up that follows it.
    pointer_cancelled: AtomicBool,
    cancel_up: AtomicBool,
    sizes: AtomicU32,
    focus: AtomicU32,
    resizes: AtomicU32,
    // Resizes with a size that came after a `WindowSize` the game sent (not a size request), once armed.
    task_resizes: AtomicU32,
    minimised: AtomicBool,
    // The last `WindowSize` was the restore from minimised.
    restore: AtomicBool,
    last_size_injected: AtomicBool,
    injected_size: AtomicBool,
}

impl WindowChecks {
    /// Lines of each kind (size, focus, resize) a run logs; later ones are counted only.
    pub const LOG_LIMIT: u32 = 50;
    /// Resizes of the in-game task that `Step::Resized` needs.
    pub const TASK_RESIZES: u32 = 2;

    pub const fn new(progress: &'static Progress) -> Self {
        Self {
            progress,
            armed: AtomicBool::new(false),
            pointer_cancelled: AtomicBool::new(false),
            cancel_up: AtomicBool::new(false),
            sizes: AtomicU32::new(0),
            focus: AtomicU32::new(0),
            resizes: AtomicU32::new(0),
            task_resizes: AtomicU32::new(0),
            minimised: AtomicBool::new(false),
            restore: AtomicBool::new(false),
            last_size_injected: AtomicBool::new(false),
            injected_size: AtomicBool::new(false),
        }
    }

    /// Forgets every count and flag.
    pub fn reset(&self) {
        for flag in [
            &self.armed,
            &self.pointer_cancelled,
            &self.cancel_up,
            &self.minimised,
            &self.restore,
            &self.last_size_injected,
            &self.injected_size,
        ] {
            flag.store(false, Ordering::Release);
        }
        for count in [&self.sizes, &self.focus, &self.resizes, &self.task_resizes] {
            count.store(0, Ordering::Release);
        }
    }

    /// The overlay checks are done; the window events from now on are the in-game task's.
    pub fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }

    pub fn is_armed(&self) -> bool {
        self.armed.load(Ordering::Acquire)
    }

    /// Whether an injected `WindowSize` (from a size request) came since the last call; clears it.
    pub fn take_injected_size(&self) -> bool {
        self.injected_size.swap(false, Ordering::AcqRel)
    }

    /// Whether the window steps of the in-game task were all seen.
    pub fn task_done(&self) -> bool {
        [Step::WindowSize, Step::Resized, Step::WindowPointer, Step::WindowChar]
            .into_iter()
            .all(|step| self.progress.seen(step))
    }

    /// `WindowPointer`: the posted left button down. Once armed, that click and its up are cancelled, so
    /// the game screen behind it does not take it.
    pub fn pointer(&self, log: Logger, event: &mut WindowPointer) {
        if event.kind() != PointerKind::Button || event.button() != Some(MouseButton::Left) || event.is_injected() {
            return;
        }
        if !event.down() {
            if self.cancel_up.swap(false, Ordering::AcqRel) {
                event.cancel();
            }
            return;
        }
        let armed = self.is_armed();
        if armed && self.progress.set(Step::WindowPointer) {
            let client = event.client();
            let buffer = event.buffer();
            log.info(
                "window pointer",
                &[
                    ("button", "Left"),
                    ("down", "true"),
                    ("client", &Self::point_text(client.x, client.y)),
                    ("buffer", &Self::point_text(buffer.x, buffer.y)),
                ],
            );
        }
        if armed && !self.pointer_cancelled.swap(true, Ordering::AcqRel) {
            event.cancel();
            self.cancel_up.store(true, Ordering::Release);
            log.info("window pointer cancelled", &[("button", "Left")]);
        }
    }

    /// `WindowChar`: the posted `a`, cancelled once armed.
    pub fn char(&self, log: Logger, event: &mut WindowChar) {
        if event.ch() != Some('a') || !self.is_armed() {
            return;
        }
        event.cancel();
        if self.progress.set(Step::WindowChar) {
            log.info("window char", &[("ch", "a")]);
        }
    }

    /// `WindowSize`: the sizes the window got, and the first real one of the in-game task.
    pub fn size(&self, log: Logger, event: &WindowSize) {
        let injected = event.is_injected();
        let minimised = event.minimised();
        let size = event.size();
        self.last_size_injected.store(injected, Ordering::Release);
        if injected {
            self.injected_size.store(true, Ordering::Release);
        }
        // The last size before this one was a minimise: this one is the restore, and only the resize that follows it is
        // the resize after the restore (the game may not resize when the size did not change)
        let was_minimised = self.minimised.swap(minimised, Ordering::AcqRel);
        let restore = !minimised && was_minimised;
        self.restore.store(restore, Ordering::Release);
        let count = self.sizes.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if count <= Self::LOG_LIMIT {
            log.info(
                "window size",
                &[
                    ("size", &Self::size_text(size)),
                    ("minimised", Self::bool_text(minimised)),
                    ("injected", Self::bool_text(injected)),
                ],
            );
        }
        if restore {
            log.info("window restored", &[("size", &Self::size_text(size))]);
        }
        if !injected && !minimised && self.is_armed() && self.progress.set(Step::WindowSize) {
            log.info("window size seen", &[("size", &Self::size_text(size))]);
        }
    }

    /// `WindowFocus`: the gains and losses, logged up to `LOG_LIMIT`.
    pub fn focus(&self, log: Logger, event: &WindowFocus) {
        let count = self.focus.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if count <= Self::LOG_LIMIT {
            log.info(
                "window focus",
                &[("gained", Self::bool_text(event.gained())), ("replayed", Self::bool_text(event.replayed()))],
            );
        }
    }

    /// `SwapChainResized`: each resize, the one after a restore from minimised, and the in-game task's.
    pub fn resized(&self, log: Logger, event: &SwapChainResized) {
        let size = Size { width: event.width, height: event.height };
        let count = self.resizes.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if event.ok == 0 {
            log.warn("resized", &[("ok", "false"), ("size", &Self::size_text(size))]);
            return;
        }
        let has_size = size.width != 0 && size.height != 0;
        let restored = has_size && self.restore.swap(false, Ordering::AcqRel);
        if restored {
            log.info("resized", &[("ok", "true"), ("size", &Self::size_text(size)), ("after", "restore")]);
        } else if count <= Self::LOG_LIMIT {
            log.info("resized", &[("ok", "true"), ("size", &Self::size_text(size))]);
        }
        if has_size && self.is_armed() && !self.last_size_injected.load(Ordering::Acquire) {
            let task = self.task_resizes.fetch_add(1, Ordering::AcqRel).saturating_add(1);
            if task >= Self::TASK_RESIZES && self.progress.set(Step::Resized) {
                log.info("resizes seen", &[("count", &task.to_string())]);
            }
        }
    }

    /// `Shutdown`: whether the game still runs and the event has a client.
    pub fn shutdown(&self, log: Logger, at: &EventCtx) {
        let client = if at.client().is_some() { "yes" } else { "none" };
        log.info("shutdown", &[("game_alive", Self::bool_text(at.game_alive)), ("client", client)]);
    }

    fn size_text(size: Size) -> String {
        format!("{}x{}", size.width, size.height)
    }

    fn point_text(x: f32, y: f32) -> String {
        format!("{x:.0},{y:.0}")
    }

    fn bool_text(value: bool) -> &'static str {
        if value { "true" } else { "false" }
    }
}

use std::cell::Cell;

thread_local! {
    // How many calls of the game window's own procedure, through the loader's, run on this thread now.
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Whether the current thread is inside the game window's own procedure, called through the loader's window
/// procedure. The key feed reads it to log where a real key comes from.
pub struct WindowProcDepth;

impl WindowProcDepth {
    /// Runs `body` (a call of the game's procedure) counted as inside the window procedure.
    pub fn run<R>(body: impl FnOnce() -> R) -> R {
        DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
        let result = body();
        DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
        result
    }

    /// Whether a call of the game's procedure through the loader's runs on this thread.
    pub fn inside() -> bool {
        DEPTH.with(|depth| depth.get() > 0)
    }
}

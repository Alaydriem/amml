use std::cell::Cell;

use crate::Callback;

thread_local! {
    static CURRENT: Cell<Option<(usize, Callback)>> = const { Cell::new(None) };
}

/// The mod whose callback the current thread is inside, and which callback it is, so that `hook`
/// and `unhook` know which mod the caller is and whether a hook is allowed.
pub struct CurrentMod;

impl CurrentMod {
    /// Runs `body` with `index` as the current mod and `callback` as its callback.
    pub fn with<R>(index: usize, callback: Callback, body: impl FnOnce() -> R) -> R {
        let previous = CURRENT.with(|cell| cell.replace(Some((index, callback))));
        let result = body();
        CURRENT.with(|cell| cell.set(previous));
        result
    }

    pub fn get() -> Option<(usize, Callback)> {
        CURRENT.with(Cell::get)
    }
}

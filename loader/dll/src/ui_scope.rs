use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_runtime::{UiScopeKind, UiScopeStack};

// The serial the next scope gets; 0 is kept for "no scope".
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(1);

thread_local! {
    // The scopes running on this thread.
    static STACK: RefCell<UiScopeStack> = const { RefCell::new(UiScopeStack::new()) };
    // The render context of each running scope, innermost last; 0 for a scope that has none.
    static CONTEXTS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    // The serial of each running scope, innermost last.
    static SERIALS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// A running UI scope on this thread: from `enter` until the value drops. A table call checks the innermost
/// scope, so it answers only inside the dispatch that opened it, on that thread.
pub struct UiScope {
    // Whether `enter` pushed an entry that the drop pops.
    pushed: bool,
}

impl UiScope {
    /// Opens a scope of `kind` that has no render context.
    pub fn enter(kind: UiScopeKind) -> Self {
        Self::enter_with_context(kind, 0)
    }

    /// Opens a scope of `kind` whose draw calls go to the render context at address `context`.
    pub fn enter_with_context(kind: UiScopeKind, context: usize) -> Self {
        let pushed = STACK.try_with(|stack| stack.borrow_mut().push(kind)).is_ok();
        if pushed {
            let _ = CONTEXTS.try_with(|contexts| contexts.borrow_mut().push(context));
            let serial = NEXT_SERIAL.fetch_add(1, Ordering::Relaxed);
            let _ = SERIALS.try_with(|serials| serials.borrow_mut().push(serial));
        }
        Self { pushed }
    }

    /// The innermost running scope of this thread.
    pub fn current() -> Option<UiScopeKind> {
        STACK.try_with(|stack| stack.borrow().current()).ok().flatten()
    }

    /// Whether the innermost running scope of this thread is the `ScreenRender` dispatch of `view`.
    pub fn runs_view(view: u64) -> bool {
        STACK.try_with(|stack| stack.borrow().runs_view(view)).unwrap_or(false)
    }

    /// The serial of the innermost running scope of this thread, unique for each opened scope; 0 when none runs.
    pub fn serial() -> u64 {
        SERIALS.try_with(|serials| serials.borrow().last().copied()).ok().flatten().unwrap_or(0)
    }

    /// The render context of the innermost running scope of this thread; `None` when it has none.
    pub fn context() -> Option<usize> {
        CONTEXTS.try_with(|contexts| contexts.borrow().last().copied()).ok().flatten().filter(|context| *context != 0)
    }
}

impl Drop for UiScope {
    fn drop(&mut self) {
        if self.pushed {
            let _ = STACK.try_with(|stack| stack.borrow_mut().pop());
            let _ = CONTEXTS.try_with(|contexts| contexts.borrow_mut().pop());
            let _ = SERIALS.try_with(|serials| serials.borrow_mut().pop());
        }
    }
}

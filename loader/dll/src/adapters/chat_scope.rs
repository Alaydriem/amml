use std::cell::RefCell;

use crate::ChatEvent;

thread_local! {
    // The chat events running on this thread, innermost last.
    static STACK: RefCell<Vec<ChatEvent>> = const { RefCell::new(Vec::new()) };
}

/// A running `ChatMessageAdded` on this thread: from `enter` until `finish` or the drop. The chat calls of the `ui` table
/// answer only inside the dispatch that opened the scope, on that thread.
pub struct ChatScope {
    // Whether `enter` pushed an entry that `finish` or the drop pops.
    pushed: bool,
}

impl ChatScope {
    /// Opens a scope for `event`.
    pub fn enter(event: ChatEvent) -> Self {
        let pushed = STACK.try_with(|stack| stack.borrow_mut().push(event)).is_ok();
        Self { pushed }
    }

    /// Runs `body` on the innermost running event of this thread; `None` when none runs.
    pub fn with<R>(body: impl FnOnce(&mut ChatEvent) -> R) -> Option<R> {
        STACK.try_with(|stack| stack.borrow_mut().last_mut().map(body)).ok().flatten()
    }

    /// Whether a chat event runs on this thread.
    pub fn is_running() -> bool {
        STACK.try_with(|stack| !stack.borrow().is_empty()).unwrap_or(false)
    }

    /// Closes the scope and gives its event, with the edits the listeners made.
    pub fn finish(mut self) -> Option<ChatEvent> {
        if !self.pushed {
            return None;
        }
        self.pushed = false;
        STACK.try_with(|stack| stack.borrow_mut().pop()).ok().flatten()
    }
}

impl Drop for ChatScope {
    fn drop(&mut self) {
        if self.pushed {
            let _ = STACK.try_with(|stack| stack.borrow_mut().pop());
        }
    }
}

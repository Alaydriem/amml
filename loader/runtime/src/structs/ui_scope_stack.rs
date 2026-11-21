use crate::structs::UiScopeKind;

/// The UI scopes that run, innermost last. A table call checks the innermost one. The value holds no thread
/// state: the loader keeps one stack for each thread.
#[derive(Debug, Default)]
pub struct UiScopeStack {
    scopes: Vec<UiScopeKind>,
}

impl UiScopeStack {
    pub const fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Opens a scope of `kind`.
    pub fn push(&mut self, kind: UiScopeKind) {
        self.scopes.push(kind);
    }

    /// Closes the innermost scope.
    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    /// The innermost scope.
    pub fn current(&self) -> Option<UiScopeKind> {
        self.scopes.last().copied()
    }

    /// Whether the innermost scope is the `ScreenRender` dispatch of `view`.
    pub fn runs_view(&self, view: u64) -> bool {
        matches!(self.current(), Some(UiScopeKind::ScreenRender { view: running }) if running == view)
    }
}

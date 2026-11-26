use amml_abi::WindowFocus;

/// Typed reads of a `WindowFocus`.
pub trait WindowFocusExt {
    /// True for a gain, false for a loss.
    fn gained(&self) -> bool;
    /// Whether this loss is one a listener held back earlier, passed on by `Window::replay_focus_loss`.
    fn replayed(&self) -> bool;
}

impl WindowFocusExt for WindowFocus {
    fn gained(&self) -> bool {
        self.gained != 0
    }

    fn replayed(&self) -> bool {
        self.replayed != 0
    }
}

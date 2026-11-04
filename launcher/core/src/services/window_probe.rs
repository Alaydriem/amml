// Asks whether a process shows a window.
pub trait WindowProbe {
    fn has_window(&self, pid: u32) -> bool;
}

/// One window message about focus: `WM_KILLFOCUS`, `WM_ACTIVATE` or `WM_ACTIVATEAPP`, as the window procedure got it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusMessage {
    pub msg: u32,
    pub wparam: usize,
    pub lparam: isize,
}

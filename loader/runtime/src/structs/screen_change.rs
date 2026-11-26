/// A change of the top screen: the screen id before and after. 0 is no screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChange {
    pub old: u64,
    pub new: u64,
}

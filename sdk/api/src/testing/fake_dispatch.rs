/// The result of one `FakeGame::raise`: whether a listener cancelled the event, and whether one
/// panicked (the loader would then disable its mod).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeDispatch {
    pub cancelled: bool,
    pub panicked: bool,
}

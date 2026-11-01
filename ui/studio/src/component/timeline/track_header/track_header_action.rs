/// What a click on a track header asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackHeaderAction {
    Select,
    ToggleHidden,
    ToggleLocked,
}

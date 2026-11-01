/// A control of the top bar that has a place on the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopBarControl {
    SceneChip,
    Undo,
    Redo,
    UndoHistory,
    Settings,
    Close,
    Coverage,
    Queue,
    Export,
}

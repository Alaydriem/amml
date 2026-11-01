/// What a click in the top bar asks for. One variant per control on Paper `G-0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarAction {
    OpenSceneSwitcher,
    Undo,
    Redo,
    OpenUndoHistory,
    OpenSettings,
    CloseReplay,
    OpenCoverage,
    OpenQueue,
    OpenExport,
}

use egui::Pos2;

/// What the render queue asks for. Job indexes are into `RenderQueueView::jobs`. The row's trailing button cancels a
/// job that is not over and removes one that is; a right click on a row asks for the job menu (Cancel, Retry, Move
/// up, Move down, Remove), which the host opens at the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderQueueAction {
    Retry(usize),
    ShowFile(usize),
    Cancel(usize),
    Remove(usize),
    OpenJobMenu(usize, Pos2),
    ClearFinished,
    PauseAll,
    ResumeAll,
    /// The Close button, the close mark, Escape or a click outside.
    Close,
}

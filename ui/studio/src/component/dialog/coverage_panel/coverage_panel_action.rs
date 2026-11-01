use super::CoverageFilter;

/// What the coverage panel asks for. `GoTo` indexes are into `CoveragePanelView::groups` and that group's rows; the
/// host then shows the item and rings it with the amml-ui focus ring (Paper `42V-0`, "Go to").
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoveragePanelAction {
    SetFilter(CoverageFilter),
    GoTo {
        group: usize,
        row: usize,
    },
    /// The close mark, Escape or a click outside.
    Close,
}
